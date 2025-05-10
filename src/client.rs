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
        let response = self.client.get(&data.url).send().await.context(format!(
            "Failed to call Name({}), URL({})",
            data.name, data.url
        ))?;

        let status_code = response.status();
        let text = response.text().await.context(format!(
            "Return body from Name({}), URL({}) is corrupted",
            data.name, data.url,
        ))?;

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
        let left_json = Self::to_pretty_json(&self.text).unwrap_or(self.text.clone());
        let right_json = Self::to_pretty_json(&other.text).unwrap_or(other.text.clone());
        format!(
            "{}:{}|{}:{}\n{:?}",
            self.name,
            self.url,
            other.name,
            other.url,
            diff_lines(&left_json, &right_json).prettytable()
        )
    }

    fn to_pretty_json(str: &str) -> Result<String> {
        let json: Value = serde_json::from_str(str)?;
        Ok(serde_json::to_string_pretty(&json)?)
    }
}
