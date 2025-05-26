use serde::Deserialize;
use std::fmt::Display;

#[derive(Deserialize)]
pub struct RequestsConfig {
    pub name: String,
    pub left: PartRequestConfig,
    pub right: PartRequestConfig,
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(default)]
    pub ignore_lines: Vec<String>,
    pub requests: Vec<RequestsConfig>,
}

impl Config {
    pub fn requires_cache(&self) -> bool {
        self.requests
            .iter()
            .any(|r| r.left.cached || r.right.cached)
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
    #[serde(default)]
    pub ignore_lines: Vec<String>,
    #[serde(default)]
    pub cached: bool,
    pub user: Option<String>,
    pub password: Option<String>,
    pub token: Option<String>,
}
