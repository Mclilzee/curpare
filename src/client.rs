use std::fmt::Display;

use anyhow::{Context, Result};
use serde_json::Value;

use crate::meta_data::PartRequestConfig;
use crate::meta_data::PartResponse;
use crate::meta_data::RequestsConfig;
use crate::meta_data::Response;

#[derive(Clone)]
pub struct Client {
    inner: reqwest::Client,
}

impl Client {
    pub fn new() -> Self {
        Self {
            inner: reqwest::Client::new(),
        }
    }

    pub async fn get_response(&self, requests: RequestsConfig) -> Result<Response> {
        let left_response = self.get(requests.left).await?;
        let right_response = self.get(requests.right).await?;

        Ok(Response::new(requests.name, left_response, right_response))
    }

    async fn get(&self, part_request: PartRequestConfig) -> Result<PartResponse> {
        let mut request = self.inner.get(&part_request.url);
        if let Some(user) = &part_request.user {
            request = request.basic_auth(user, part_request.password.as_ref());
        }

        if let Some(token) = &part_request.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        let status_code = response.status();
        let mut text = response.text().await?;

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
