mod request;
mod response;

use std::{
    collections::HashMap,
    fs::{OpenOptions, create_dir_all},
    io::{BufReader, Write},
    mem,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::{Context, Result, anyhow};
pub use request::{Config, RequestsConfig};
use reqwest::{
    Method,
    header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue},
};
use response::PartResponse;
pub use response::Response;
use serde_json::Value;

use crate::client::request::PartRequestConfig;

#[derive(Clone)]
pub struct Client {
    reqwest: reqwest::Client,
    cache: Arc<Mutex<HashMap<String, PartResponse>>>,
    cache_location: Option<PathBuf>,
}

impl Client {
    pub fn new() -> Self {
        Self {
            reqwest: reqwest::Client::new(),
            cache: Arc::new(Mutex::new(HashMap::new())),
            cache_location: None,
        }
    }

    pub async fn get_response(&mut self, request: &RequestsConfig) -> Result<Response> {
        let (left_response, right_response) =
            tokio::join!(self.get(&request.left), self.get(&request.right));

        let mut left_response = left_response?;
        let mut right_response = right_response?;

        if request.left.cached || request.right.cached {
            {
                let mut cache = self.cache.lock().unwrap();
                if request.left.cached {
                    cache.insert(left_response.url.clone(), left_response.clone());
                }

                if request.right.cached {
                    cache.insert(right_response.url.clone(), right_response.clone());
                }
            }
        }

        if !request.left.ignore_lines.is_empty() {
            left_response.text = Self::filter(&left_response.text, &request.left.ignore_lines);
        }

        if !request.right.ignore_lines.is_empty() {
            right_response.text = Self::filter(&right_response.text, &request.left.ignore_lines);
        }

        Ok(Response::new(request.name.clone(), left_response, right_response))
    }

    async fn get(&self, request: &PartRequestConfig) -> Result<PartResponse> {
        if request.cached
            && let Some(response) = self.cache.lock().unwrap().get(&request.url)
        {
            return Ok(response.clone());
        }

        self.get_from_url(request).await
    }

    async fn get_from_url(&self, part_request: &PartRequestConfig) -> Result<PartResponse> {
        let method = part_request
            .method
            .as_deref()
            .unwrap_or("GET")
            .parse::<Method>()
            .map_err(|_| {
                anyhow!(
                    "Unrecognized method {}",
                    part_request
                        .method
                        .as_ref()
                        .expect("Method should exist at this point")
                )
            })?;

        let mut request = self.reqwest.request(method, &part_request.url);

        if let Some(basic_auth) = &part_request.basic_auth {
            request = request.basic_auth(&basic_auth.username, basic_auth.password.clone());
        }

        let headers = part_request
            .headers
            .iter()
            .map(|(k, v)| {
                (
                    HeaderName::from_bytes(k.as_bytes())
                        .expect("Header contains invalid UTF-8 characters"),
                    HeaderValue::from_str(v).expect("Header value is not valid"),
                )
            })
            .collect::<HeaderMap>();

        let response = request
            .headers(headers)
            .query(&part_request.query)
            .send()
            .await
            .with_context(|| format!("Failed sending request to URL {}", part_request.url))?;

        let status_code = response.status();

        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .cloned()
            .ok_or_else(|| anyhow!("CONTENT_TYPE header not found"))?;

        let content_type = content_type.to_str().map_err(|e| anyhow::anyhow!(e))?;

        let mut text = response.text().await.map_err(|e| anyhow::anyhow!(e))?;
        text = match content_type {
            ct if ct.starts_with("application/json") => Self::json_pretty_format(&text)
                .with_context(|| format!("Failed to format JSON for URL: {}", part_request.url))?,
            ct => {
                return Err(anyhow!(
                    "Could not format response of content_type: {ct}\nURL: {}",
                    part_request.url
                ));
            }
        };

        Ok(PartResponse::new(
            part_request.url.clone(),
            status_code,
            text,
        ))
    }

    fn json_pretty_format(text: &str) -> Result<String> {
        serde_json::from_str::<Value>(text)
            .and_then(|value| serde_json::to_string_pretty(&value))
            .context("Invalid body format, expecting JSON format")
    }

    fn filter(text: &str, ignore_list: &[String]) -> String {
        text.lines()
            .filter(|&line| Self::ignore_line(line, ignore_list))
            .collect::<Vec<&str>>()
            .join("\n")
    }

    fn ignore_line(line: &str, ignore_lines: &[String]) -> bool {
        !ignore_lines.iter().any(|ignore| line.contains(ignore))
    }

    pub fn load_cache(&mut self, cache_location: PathBuf) -> Result<()> {
        if let Some(parent) = cache_location.parent() {
            let _ = create_dir_all(parent).with_context(|| {
                format!(
                    "Failed to create sub directories for path {}",
                    cache_location.display()
                )
            });
        }

        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&cache_location)
            .with_context(|| {
                format!(
                    "Failed to open file for reading cache for path {}",
                    cache_location.display()
                )
            })?;

        let reader = BufReader::new(&file);
        let cache: HashMap<String, PartResponse> =
            serde_json::from_reader(reader).unwrap_or_default();

        self.cache = Arc::new(Mutex::new(cache));
        self.cache_location = Some(cache_location);
        Ok(())
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        if self.cache_location.is_none() || Arc::strong_count(&self.cache) > 1 {
            return;
        }

        let cache_location = self.cache_location.take().unwrap();
        let cache = mem::replace(&mut self.cache, Arc::new(Mutex::new(HashMap::new())));
        let cache = Arc::try_unwrap(cache)
            .ok()
            .expect("There should only be one reference to cache at this point")
            .into_inner()
            .expect("There should be one reference to this mutex at this point");

        let cache_json = serde_json::to_vec(&cache).expect("To unwrap");
        match OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&cache_location)
        {
            Ok(mut file) => {
                let result = file.write_all(&cache_json);
                if let Err(e) = result {
                    eprintln!(
                        "Failed to save new cache into cache file for path {}: {e:?}",
                        cache_location.display()
                    );
                }
            }
            Err(e) => eprintln!(
                "Failed to open file for saving new cache for path {}: {e:?}",
                cache_location.display()
            ),
        }
    }
}
