mod request;
mod response;

use std::{
    collections::HashMap,
    fs::{OpenOptions, create_dir_all},
    io::{BufReader, Write},
    path::PathBuf,
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
    cache: HashMap<String, PartResponse>,
    cache_location: Option<PathBuf>,
}

impl Client {
    pub fn new() -> Self {
        Self {
            reqwest: reqwest::Client::new(),
            cache: HashMap::new(),
            cache_location: None,
        }
    }

    pub async fn get_response(&self, requests: RequestsConfig) -> Result<Response> {
        let (left_response, right_response) =
            tokio::join!(self.get(requests.left), self.get(requests.right));

        let left_response = left_response?;
        let right_response = right_response?;

        Ok(Response::new(requests.name, left_response, right_response))
    }

    async fn get(&self, request: PartRequestConfig) -> Result<PartResponse> {
        if request.cached
            && let Some(response) = self.cache.get(&request.url)
        {
            return Ok(response.clone());
        }

        self.get_from_url(request).await
    }

    async fn get_from_url(&self, part_request: PartRequestConfig) -> Result<PartResponse> {
        let method = part_request
            .method
            .as_deref()
            .unwrap_or("GET")
            .parse::<Method>()
            .map_err(|_| {
                anyhow!(
                    "Unrecognized method {}",
                    part_request.method.unwrap_or_default()
                )
            })?;

        let mut request = self.reqwest.request(method, &part_request.url);

        if let Some(basic_auth) = part_request.basic_auth {
            request = request.basic_auth(basic_auth.username, basic_auth.password);
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

        if !part_request.ignore_lines.is_empty() {
            text = Self::filter(&text, &part_request.ignore_lines);
        }

        Ok(PartResponse::new(part_request.url, status_code, text))
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

        self.cache = cache;
        self.cache_location = Some(cache_location);
        Ok(())
    }

    pub fn cache_response(&mut self, response: &PartResponse) {
        self.cache.insert(response.url.clone(), response.clone());
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        if self.cache_location.is_some() {
            return;
        }

        let cache_location = self.cache_location.take().unwrap();
        let cache_json = serde_json::to_vec(&self.cache).expect("To unwrap");
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&cache_location)
            .unwrap_or_else(|e| {
                panic!(
                    "Failed to open file for saving new cache for path {}: {e:?}",
                    cache_location.display()
                )
            })
            .write_all(&cache_json)
            .unwrap_or_else(|e| {
                panic!(
                    "Failed to save new cache into cache file for path {}: {e:?}",
                    cache_location.display()
                )
            });
    }
}
