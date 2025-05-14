use serde::Deserialize;
use std::fmt::Display;

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
