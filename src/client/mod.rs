mod clients;
mod meta_data;

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use clients::CachelesClient;
use meta_data::{PartRequestConfig, PartResponse};
pub use meta_data::{RequestsConfig, Response};
use serde_json::Value;

#[derive(Clone)]
pub struct Client {
    client: 

}

impl Client {
    pub fn new() -> Self {
        Self { CachelessClient(CachelesClient::new()) }
    }

    pub fn new_cached(cache_location: PathBuf) -> Result<Self> {
        todo!();
    }

    pub async fn get_response(&self, requests: RequestsConfig) -> Result<Response> {
        let left_response = self.get(requests.left).await?;
        let right_response = self.get(requests.right).await?;

        Ok(Response::new(requests.name, left_response, right_response))
    }

    async fn get(&self, part_request: PartRequestConfig) -> Result<PartResponse> {
        self.get_from_url(part_request).await
    }

    async fn get_from_url(&self, part_request: PartRequestConfig) -> Result<PartResponse> {
        let mut request = self.client.get(&part_request.url);
        if let Some(user) = &part_request.user {
            request = request.basic_auth(user, part_request.password.as_ref());
        }

        if let Some(token) = &part_request.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await.context(format!(
            "Failed sending request to URL {}",
            part_request.url
        ))?;

        let status_code = response.status();
        let mut text = response.text().await.context(format!(
            "Failed extracting body for URL {}",
            part_request.url
        ))?;

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
