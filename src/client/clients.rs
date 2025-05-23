use std::{collections::HashMap, fs::OpenOptions, io::Write, path::PathBuf};

use super::{
    request::{PartRequestConfig, RequestsConfig},
    response::{PartResponse, Response},
};
use anyhow::{Context, Result};
use serde_json::Value;

pub trait RequestClient {
    async fn get_response(&mut self, requests: RequestsConfig) -> Result<Response>;
    fn get_client(&self) -> &reqwest::Client;

    async fn get_from_url(&self, part_request: PartRequestConfig) -> Result<PartResponse> {
        let mut request = self.get_client().get(&part_request.url);
        if let Some(user) = &part_request.user {
            request = request.basic_auth(user, part_request.password.as_ref());
        }

        if let Some(token) = &part_request.token {
            request = request.bearer_auth(token);
        }

        let response = request
            .send()
            .await
            .with_context(|| format!("Failed sending request to URL {}", part_request.url))?;

        let status_code = response.status();
        let mut text = response
            .text()
            .await
            .map_err(|e| anyhow::anyhow!(e))
            .and_then(|s| Self::pretty_format(&s))
            .with_context(|| format!("Failed extracting body for URL {}", part_request.url))?;

        if let Some(ignore_lines) = part_request.ignore_lines.as_ref() {
            text = Self::filter(text, ignore_lines);
        }

        Ok(PartResponse::new(part_request.url, status_code, text))
    }

    fn pretty_format(text: &str) -> Result<String> {
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
