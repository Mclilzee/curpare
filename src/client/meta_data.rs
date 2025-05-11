use std::fmt::Display;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct RequestsConfig {
    pub name: String,
    pub left: PartRequestConfig,
    pub right: PartRequestConfig,
}

impl RequestsConfig {
    pub fn requires_cache(&self) -> bool {
        self.left.cached || self.right.cached
    }
}

impl Display for RequestsConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} => {}", self.name, self.left.url, self.right.url)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartRequestConfig {
    pub url: String,
    pub ignore_lines: Option<Vec<String>>,
    pub cached: bool,
    pub user: Option<String>,
    pub password: Option<String>,
    pub token: Option<String>,
}

pub struct Response {
    pub name: String,
    left: PartResponse,
    right: PartResponse,
}

impl Response {
    pub fn new(name: String, left: PartResponse, right: PartResponse) -> Self {
        Self { name, left, right }
    }

    pub fn diff(&self) -> String {
        format!(
            "{}: {} => {}\n{}",
            self.name, self.left.url, self.right.url, self.left
        )
    }
}

#[derive(Clone)]
pub struct PartResponse {
    pub url: String,
    pub status_code: u16,
    pub text: String,
}

impl PartResponse {
    pub fn new(url: String, status_code: reqwest::StatusCode, text: String) -> Self {
        Self {
            url,
            status_code: status_code.into(),
            text,
        }
    }
}

impl Display for PartResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Status code {}\n{}", self.status_code, self.text)
    }
}
