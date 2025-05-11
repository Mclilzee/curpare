use std::fmt::Display;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct RequestsConfig {
    pub left: PartRequestConfig,
    pub right: PartRequestConfig,
}

impl Display for RequestsConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Left: {}, Right: {}", self.left, self.right)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartRequestConfig {
    pub name: String,
    pub url: String,
    pub ignore_lines: Option<Vec<String>>,
    pub cached: Option<bool>,
    pub user: Option<String>,
    pub password: Option<String>,
    pub token: Option<String>,
}

impl Display for PartRequestConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Name({}), URL({})", self.name, self.url)
    }
}

pub struct Response {
    left: PartResponse,
    right: PartResponse,
}

impl Response {
    pub fn new(left: PartResponse, right: PartResponse) -> Self {
        Self { left, right }
    }

    pub fn diff(&self) -> String {
        format!(
            "{}@{} => {}@{}\n{}",
            self.left.name, self.left.url, self.right.name, self.right.url, self.left.text
        )
    }
}

pub struct PartResponse {
    pub name: String,
    pub url: String,
    pub status_code: u16,
    pub text: String,
}

impl PartResponse {
    pub fn new(name: String, url: String, status_code: reqwest::StatusCode, text: String) -> Self {
        Self {
            name,
            url,
            status_code: status_code.into(),
            text,
        }
    }
}
