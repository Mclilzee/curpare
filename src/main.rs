#![warn(clippy::all, clippy::pedantic)]

mod args;
mod client;

use std::{
    path::{Path, PathBuf},
    process,
    sync::Arc,
};

use anyhow::{Context, Result};
use args::Args;
use clap::Parser;
use client::{Client, RequestsConfig, Response};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let meta_data = get_meta_data(&args);
    let client = if meta_data.iter().any(|config| config.requires_cache()) {
        let cache_location = Path::new("./cache").join(args.path.file_name().unwrap());
        if args.clear_cache {
            meta_data = meta_data
                .into_iter()
                .map(|meta_data| meta_data.with_cache())
                .collect();
        }
        Client::new_cached(cache_location).context("Failed to load cache")?
    } else {
        Client::new()
    };

    get_responses(client, meta_data)
        .await
        .iter()
        .map(|response| response.diff().to_string())
        .for_each(|s| println!("{s}"));

    Ok(())
}

async fn get_responses(client: Client, meta_data: Vec<RequestsConfig>) -> Vec<Response> {
    let mut handles = vec![];
    let client = Arc::new(Mutex::new(client));
    for request in meta_data {
        let moved_client = client.clone();
        let handle = tokio::spawn({
            async move {
                println!("Sending request for {request}");
                moved_client.lock().await.get_response(request).await
            }
        });

        handles.push(handle);
    }

    let mut responses = vec![];
    for handle in handles {
        let result = handle.await.expect("Failed to unlock ansync handle");
        match result {
            Ok(response) => responses.push(response),
            Err(e) => eprintln!("{e}"),
        }
    }

    println!("All requests has been processed");
    println!("================================================");
    responses
}

fn get_meta_data(args: &Args) -> Result<Vec<RequestsConfig>> {
    let json = std::fs::read_to_string(&args.path).expect("Failed to read json file");
    let mut meta_data: Vec<RequestsConfig> = serde_json::from_str(&json)
        .with_context(|| format!("Json in path {:?} is not formatted correctly", args.path))?;

    if args.skip_ignore {
        meta_data = meta_data
            .into_iter()
            .map(|meta_data| meta_data.without_ignores())
            .collect();
    }

    if args.all_cache {
        meta_data = meta_data
            .into_iter()
            .map(|meta_data| meta_data.with_cache())
            .collect();
    } else if args.no_cache {
        meta_data = meta_data
            .into_iter()
            .map(|meta_data| meta_data.without_cache())
            .collect();
    }

    Ok(meta_data)
}
