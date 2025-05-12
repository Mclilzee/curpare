mod clients;
mod meta_data;

use std::{
    collections::HashMap,
    fs::{self, File, create_dir_all, exists},
    io::{BufReader, Read},
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result, bail};
use clients::{CachedClient, CachelesClient, RequestClient};
use meta_data::{PartRequestConfig, PartResponse};
pub use meta_data::{RequestsConfig, Response};
use serde::{Deserialize, Deserializer};
use serde_json::Value;

pub enum Client {
    CachelessClient(CachelesClient),
    CachedClient(CachedClient),
}

impl Client {
    pub fn new() -> Self {
        Self::CachelessClient(CachelesClient::new())
    }

    pub fn new_cached(cache_location: PathBuf) -> Result<Self> {
        if let Some(parent) = cache_location.parent() {
            let _ = create_dir_all(parent).with_context(|| {
                format!("Failed to create sub directories for path {cache_location:?}")
            });
        }

        let file = match cache_location.try_exists() {
            Ok(false) => File::create(&cache_location).with_context(|| {
                format!("Wasn't able to create file for path {:?}", cache_location)
            })?,
            Ok(true) => File::open(&cache_location).with_context(|| {
                format!("Wasn't able to open file for path {:?}", cache_location)
            })?,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Couldn't access file for path {:?}: {}",
                    cache_location,
                    e
                ));
            }
        };

        let reader = BufReader::new(&file);
        let cache: HashMap<String, PartResponse> =
            serde_json::from_reader(reader).unwrap_or_default();

        Ok(Self::CachedClient(CachedClient::new(cache, cache_location)))
    }

    pub async fn get_response(&self, requests: RequestsConfig) -> Result<Response> {
        match self {
            Client::CachedClient(client) => client.get_response(requests).await,
            Client::CachelessClient(client) => client.get_response(requests).await,
        }
    }
}
