use std::fmt::Display;

use serde::{Deserialize, Serialize};

pub struct Response {
    pub name: String,
    pub left: PartResponse,
    pub right: PartResponse,
}

impl Response {
    pub fn new(name: String, left: PartResponse, right: PartResponse) -> Self {
        Self { name, left, right }
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
