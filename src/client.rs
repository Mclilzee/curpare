use std::fmt::Display;

use anyhow::{Context, Result};
use prettydiff::diff_lines;
use serde_json::Value;

use crate::meta_data::MetaData;

#[derive(Clone)]
pub struct Client {
    client: reqwest::Client,
}

impl Client {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn get(&self, data: MetaData) -> Result<Response> {
        let response = self
            .client
            .get(&data.url)
            .send()
            .await
            .context(format!("Failed to call {data}",))?;

        let status_code = response.status();
        let text = response
            .text()
            .await
            .context(format!("Return body for {data} is corrupted"))?;

        Ok(Response::new(data.name, data.url, status_code, text))
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
        let text = serde_json::from_str::<Value>(&self.text)
            .and_then(|value| serde_json::to_string_pretty(&value))
            .unwrap_or(self.text.clone());

        write!(f, "Status Code: {}\n{})", self.status_code, text)
    }
}
