use std::fmt::Display;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct MetaData {
    pub name: String,
    pub url: String,
    pub ignore: Option<Vec<String>>,
    pub format: Option<bool>,
    pub cached: Option<bool>,
    pub headers: Option<Vec<(String, String)>>,
}

impl Display for MetaData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Name({}), URL({})", self.name, self.url)
    }
}
