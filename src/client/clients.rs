use std::{collections::HashMap, path::PathBuf, sync::Arc};

use super::{
    RequestsConfig, Response,
    meta_data::{PartRequestConfig, PartResponse},
};
use anyhow::{Context, Result};
use serde_json::Value;

pub trait RequestClient: Clone {
    async fn get_response(&self, requests: RequestsConfig) -> Result<Response>;
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
            .with_context(|| format!("Failed extracting body for URL {}", part_request.url))?;

        if let Some(ignore_lines) = part_request.ignore_lines.as_ref() {
            text = Self::filter(text, ignore_lines);
        }

        Ok(PartResponse::new(part_request.url, status_code, text))
    }

    fn filter(text: String, ignore_list: &[String]) -> String {
        serde_json::from_str::<Value>(&text)
            .and_then(|value| serde_json::to_string_pretty(&value))
            .unwrap_or(text)
            .lines()
            .filter(|&line| Self::ignore_line(line, ignore_list))
            .collect()
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
    async fn get_response(&self, requests: RequestsConfig) -> Result<Response> {
        let left_response = self.get_from_url(requests.left).await?;
        let right_response = self.get_from_url(requests.right).await?;

        Ok(Response::new(requests.name, left_response, right_response))
    }

    fn get_client(&self) -> &reqwest::Client {
        &self.client
    }
}

#[derive(Clone)]
pub struct CachedClient {
    client: reqwest::Client,
    cache: Arc<HashMap<String, PartResponse>>,
}

impl CachedClient {
    pub fn new(cache: HashMap<String, PartResponse>) -> Self {
        Self {
            client: reqwest::Client::new(),
            cache: cache.into(),
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
    async fn get_response(&self, requests: RequestsConfig) -> Result<Response> {
        let left_response = self.get(requests.left).await?;
        let right_response = self.get(requests.right).await?;

        Ok(Response::new(requests.name, left_response, right_response))
    }

    fn get_client(&self) -> &reqwest::Client {
        &self.client
    }
}
