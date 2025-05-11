mod clients;
mod meta_data;

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use clients::{CachedClient, CachelesClient, RequestClient};
use meta_data::{PartRequestConfig, PartResponse};
pub use meta_data::{RequestsConfig, Response};
use serde_json::Value;

#[derive(Clone)]
pub enum Client {
    CachelessClient(CachelesClient),
    CachedClient(CachedClient),
}

impl Client {
    pub fn new() -> Self {
        Self::CachelessClient(CachelesClient::new())
    }

    pub fn new_cached(cache_location: PathBuf) -> Result<Self> {
        Ok(Self::CachedClient(CachedClient::new(HashMap::new())))
    }

    pub async fn get_response(&self, requests: RequestsConfig) -> Result<Response> {
        match self {
            Client::CachedClient(client) => client.get_response(requests).await,
            Client::CachelessClient(client) => client.get_response(requests).await,
        }
    }
}
