use std::fmt::Display;

use anyhow::{Context, Result};
use serde_json::Value;

use crate::meta_data::PartRequestConfig;
use crate::meta_data::RequestsConfig;

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

    pub async fn get_response(&self, requests: RequestsConfig) {
        todo!()
    }

    pub async fn get(&self, part_request: PartRequestConfig) -> Result<Response> {
        let mut request = self.inner.get(&part_request.url);
        if let Some(user) = &part_request.user {
            request = request.basic_auth(user, part_request.password.as_ref());
        }

        if let Some(token) = &part_request.token {
            request = request.bearer_auth(token);
        }

        let response = request
            .send()
            .await
            .context(format!("Failed to call {part_request}",))?;

        let status_code = response.status();
        let mut text = response
            .text()
            .await
            .context(format!("Return body for {part_request} is corrupted"))?;

        if let Some(ignore_lines) = part_request.ignore_lines.as_ref() {
            text = Self::filter(text, ignore_lines);
        }

        Ok(Response::new(
            part_request.name,
            part_request.url,
            status_code,
            text,
        ))
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

pub struct Response {
    pub name: String,
    pub url: String,
    pub status_code: u16,
    pub text: String,
}

impl Response {
    pub fn new(name: String, url: String, status_code: reqwest::StatusCode, text: String) -> Self {
        Self {
            name,
            url,
            status_code: status_code.into(),
            text,
        }
    }

    pub fn diff(&self, other: &Response) -> String {
        format!(
            "{}@{} => {}@{}\n{}",
            self.name, self.url, other.name, other.url, self
        )
    }
}

impl Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Status Code: {}\n{})", self.status_code, self.text)
    }
}
