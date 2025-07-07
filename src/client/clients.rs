use std::{collections::HashMap, fs::OpenOptions, io::Write, path::PathBuf};

use super::{
    request::{PartRequestConfig, RequestsConfig},
    response::{PartResponse, Response},
};
use anyhow::{Context, Result, anyhow};
use reqwest::{
    Method,
    header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue},
};
use serde_json::Value;

pub trait RequestClient {
    async fn get_response(&mut self, requests: RequestsConfig) -> Result<Response>;
    fn get_client(&self) -> &reqwest::Client;

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

        let mut request = self.get_client().request(method, &part_request.url);

        if let Some(basic_auth) = part_request.basic_auth {
            request = request.basic_auth(basic_auth.username, basic_auth.password);
        }

        let headers = HeaderMap::from_iter(part_request.headers.iter().map(|(k, v)| {
            (
                HeaderName::from_bytes(k.as_bytes())
                    .expect("Header contains invalid UTF-8 characters"),
                HeaderValue::from_str(v).expect("Header value is not valid"),
            )
        }));

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

        let formatted = match content_type {
            ct if ct.starts_with("application/json") => {
                Self::json_pretty_format(&text).with_context(|| "Failed to format JSON")?
            }
            ct => {
                return Err(anyhow!("Could not format response: content_type: {ct}"));
            }
        };

        if !part_request.ignore_lines.is_empty() {
            text = Self::filter(formatted, &part_request.ignore_lines);
        }

        Ok(PartResponse::new(part_request.url, status_code, text))
    }

    fn json_pretty_format(text: &str) -> Result<String> {
        serde_json::from_str::<Value>(text)
            .and_then(|value| serde_json::to_string_pretty(&value))
            .context("Invalid body format, expecting JSON format")
    }

    fn filter(text: String, ignore_list: &[String]) -> String {
        text.lines()
            .filter(|&line| Self::ignore_line(line, ignore_list))
            .collect::<Vec<&str>>()
            .join("\n")
    }

    fn ignore_line(line: &str, ignore_lines: &[String]) -> bool {
        !ignore_lines.iter().any(|ignore| line.contains(ignore))
    }
}

#[derive(Clone)]
pub struct CachelesClient {
    client: reqwest::Client,
}

impl CachelesClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}
impl RequestClient for CachelesClient {
    async fn get_response(&mut self, requests: RequestsConfig) -> Result<Response> {
        let left_response = self.get_from_url(requests.left).await?;
        let right_response = self.get_from_url(requests.right).await?;

        Ok(Response::new(requests.name, left_response, right_response))
    }

    fn get_client(&self) -> &reqwest::Client {
        &self.client
    }
}

pub struct CachedClient {
    client: reqwest::Client,
    cache: HashMap<String, PartResponse>,
    cache_location: PathBuf,
}

impl CachedClient {
    pub fn new(cache: HashMap<String, PartResponse>, cache_location: PathBuf) -> Self {
        Self {
            client: reqwest::Client::new(),
            cache,
            cache_location,
        }
    }

    pub async fn get(&self, request: PartRequestConfig) -> Result<PartResponse> {
        if request.cached {
            if let Some(response) = self.cache.get(&request.url) {
                return Ok(response.clone());
            }
        }

        self.get_from_url(request).await
    }
}

impl RequestClient for CachedClient {
    async fn get_response(&mut self, requests: RequestsConfig) -> Result<Response> {
        let cache_left = requests.left.cached;
        let cache_right = requests.right.cached;
        let left_response = self.get(requests.left).await?;
        let right_response = self.get(requests.right).await?;

        if cache_left {
            self.cache
                .insert(left_response.url.clone(), left_response.clone());
        }

        if cache_right {
            self.cache
                .insert(right_response.url.clone(), right_response.clone());
        }

        Ok(Response::new(requests.name, left_response, right_response))
    }

    fn get_client(&self) -> &reqwest::Client {
        &self.client
    }
}

impl Drop for CachedClient {
    fn drop(&mut self) {
        let cache_json = serde_json::to_vec(&self.cache).expect("To unwrap");

        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.cache_location)
            .unwrap_or_else(|e| {
                panic!(
                    "Failed to open file for saving new cache for path {}: {e:?}",
                    self.cache_location.display()
                )
            })
            .write_all(&cache_json)
            .unwrap_or_else(|e| {
                panic!(
                    "Failed to save new cache into cache file for path {}: {e:?}",
                    self.cache_location.display()
                )
            });
    }
}
