use std::fmt::Display;

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetaData {
    pub name: String,
    pub url: String,
    pub ignore_lines: Option<Vec<String>>,
    pub cached: Option<bool>,
    pub user: Option<String>,
    pub password: Option<String>,
    pub token: Option<String>,
}

impl Display for MetaData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Name({}), URL({})", self.name, self.url)
    }
}
