#![warn(clippy::all, clippy::pedantic)]

mod args;
mod client;

use std::{
    fs::remove_file,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};

use anyhow::{Context, Result};
use args::Args;
use clap::Parser;
use client::{Client, RequestsConfig, Response};
use tempfile::{NamedTempFile, spooled_tempfile, tempfile};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let configs: Vec<RequestsConfig> = (&args).try_into()?;
    let requires_caching = configs.iter().any(RequestsConfig::requires_cache);
    let cache_location = get_cache_location(&args.path);
    if args.clear_cache {
        let cache_location = cache_location.as_ref().unwrap_or_else(|e| {
            panic!(
                "Failed to clear cache for path {}: {e:?}",
                args.path.display()
            )
        });
        remove_file(cache_location).with_context(|| {
            format!(
                "Failed to clear cache for path {}",
                cache_location.display()
            )
        })?;
    }

    let client = if requires_caching {
        Client::new_cached(cache_location?).context("Failed to load cache")?
    } else {
        Client::new()
    };

    get_responses(client, configs)
        .await
        .iter()
        .for_each(print_differences);

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

fn get_cache_location(path: &Path) -> Result<PathBuf> {
    Ok(Path::new("./cache").join(path.file_name().context("Failed to retreive file name")?))
}

fn print_differences(response: &Response) {
    println!(
        "{}: {} => {}",
        response.name, response.left.url, response.right.url
    );
    let mut left_file = NamedTempFile::new().expect("Failed to create temp file");
    let _ = left_file
        .write_all(response.left.to_string().as_bytes())
        .expect("Failed to write to temp file");

    let mut right_file = NamedTempFile::new().expect("Failed to create temp file");
    let _ = right_file
        .write_all(response.right.to_string().as_bytes())
        .expect("Failed to write to temp file");

    let output = Command::new("delta")
        .arg(left_file.path())
        .arg(right_file.path())
        .output()
        .expect("Failed to run delta");

    if !output.status.success() {
        eprintln!("Command Delta failed with status: {}", output.status);
        eprintln!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    } else {
        let output_str =
            String::from_utf8(output.stdout).expect("Failed to convert output to string");
        println!("{output_str}");
    }
}
