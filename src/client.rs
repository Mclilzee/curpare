use std::fmt::Display;

use anyhow::{Context, Result};
use serde_json::Value;

use crate::meta_data::MetaData;

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

    pub async fn get(&self, data: MetaData) -> Result<Response> {
        let mut request = self.inner.get(&data.url);
        if let Some(user) = &data.user {
            request = request.basic_auth(user, data.password.as_ref());
        }

        if let Some(token) = &data.token {
            request = request.bearer_auth(token);
        }

        let response = request
            .send()
            .await
            .context(format!("Failed to call {data}",))?;

        let status_code = response.status();
        let mut text = response
            .text()
            .await
            .context(format!("Return body for {data} is corrupted"))?;

        if let Some(ignore_lines) = data.ignore.as_ref() {
            text = Self::filter(text, ignore_lines);
        }

        Ok(Response::new(data.name, data.url, status_code, text))
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
