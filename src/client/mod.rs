mod clients;
mod request;
mod response;

use std::{
    collections::HashMap,
    fs::{OpenOptions, create_dir_all},
    io::BufReader,
    path::PathBuf,
};

use anyhow::{Context, Result};
use clients::{CachedClient, CachelesClient, RequestClient};
pub use request::{Requests, RequestsConfig};
use response::PartResponse;
pub use response::Response;

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
                format!(
                    "Failed to create sub directories for path {}",
                    cache_location.display()
                )
            });
        }

        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&cache_location)
            .with_context(|| {
                format!(
                    "Failed to open file for reading cache for path {}",
                    cache_location.display()
                )
            })?;

        let reader = BufReader::new(&file);
        let cache: HashMap<String, PartResponse> =
            serde_json::from_reader(reader).unwrap_or_default();

        Ok(Self::CachedClient(CachedClient::new(cache, cache_location)))
    }

    pub async fn get_response(&mut self, requests: RequestsConfig) -> Result<Response> {
        match self {
            Client::CachedClient(client) => client.get_response(requests).await,
            Client::CachelessClient(client) => client.get_response(requests).await,
        }
    }
}
