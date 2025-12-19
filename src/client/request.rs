use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Clone, Deserialize)]
pub struct RequestsConfig {
    pub name: String,
    pub left: PartRequestConfig,
    pub right: PartRequestConfig,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
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

impl From<Vec<RequestsConfig>> for Config {
    fn from(requests: Vec<RequestsConfig>) -> Self {
        Config {
            ignore_lines: vec![],
            requests,
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct RequestsConfig {
    pub name: String,
    pub left: PartRequestConfig,
    pub right: PartRequestConfig,
}

impl Display for RequestsConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} => {}", self.name, self.left.url, self.right.url)
    }
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartRequestConfig {
    pub url: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,

    #[serde(default)]
    pub cached: bool,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub basic_auth: Option<BasicAuth>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ignore_lines: Vec<String>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub query: HashMap<String, String>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BasicAuth {
    pub username: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}
