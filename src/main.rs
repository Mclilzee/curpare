#![warn(clippy::all, clippy::pedantic)]

mod args;
mod client;
mod meta_data;

use anyhow::Result;
use args::Args;
use clap::Parser;
use client::Client;
use meta_data::{RequestsConfig, Response};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let json = std::fs::read_to_string(args.path).expect("Failed to read json file");
    let meta_data: Vec<RequestsConfig> =
        serde_json::from_str(&json).expect("Json is not formatted correctly");

    get_responses(meta_data)
        .await
        .iter()
        .map(|response| response.diff())
        .for_each(|s| println!("{s}"));

    Ok(())
}

async fn get_responses(meta_data: Vec<RequestsConfig>) -> Vec<Response> {
    let client = Client::new();
    let mut handles = vec![];
    for request in meta_data {
        let moved_client = client.clone();
        let handle = tokio::spawn({
            async move {
                println!("Sending requests to {request}");
                moved_client.get_response(request).await
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
    responses
}
