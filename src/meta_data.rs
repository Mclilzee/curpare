use anyhow::Result;
use reqwest::{Response, StatusCode};
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

pub struct ContextResponse {
    pub name: String,
    pub url: String,
    pub status_code: StatusCode,
    pub text: String,
}

impl ContextResponse {
    pub fn new(name: String, url: String, status_code: StatusCode, text: String) -> Self {
        Self {
            name,
            url,
            status_code,
            text,
        }
    }
}
