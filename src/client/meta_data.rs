use std::fmt::Display;

use serde::{Deserialize, Serialize};

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

    pub fn with_cache(self) -> Self {
        Self {
            left: self.left.with_cache(),
            right: self.right.with_cache(),
            ..self
        }
    }

    pub fn without_cache(self) -> Self {
        Self {
            left: self.left.without_cache(),
            right: self.right.without_cache(),
            ..self
        }
    }

    pub fn without_ignores(self) -> Self {
        Self {
            left: self.left.no_ignores(),
            right: self.right.no_ignores(),
            ..self
        }
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
    #[serde(default)]
    pub cached: bool,
    pub user: Option<String>,
    pub password: Option<String>,
    pub token: Option<String>,
}

impl PartRequestConfig {
    fn with_cache(self) -> Self {
        Self {
            cached: true,
            ..self
        }
    }

    fn without_cache(self) -> Self {
        Self {
            cached: false,
            ..self
        }
    }

    fn no_ignores(self) -> Self {
        Self {
            ignore_lines: None,
            ..self
        }
    }
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

#[derive(Deserialize, Serialize, Clone)]
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
