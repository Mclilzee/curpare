mod args;
mod meta_data;

use anyhow::Result;
use args::Args;
use clap::Parser;
use meta_data::{ContextResponse, MetaData};
use prettydiff::diff_lines;
use reqwest::{Client, Response};
use serde::Deserialize;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let json = std::fs::read_to_string(args.path).expect("Failed to read json file");
    let meta_data: Vec<(MetaData, MetaData)> =
        serde_json::from_str(&json).expect("Json is not formatted correctly");

    let client = reqwest::Client::new();
    meta_data.into_iter().map(|(left, right)| {
        tokio::spawn({
            let moved_client = client.clone();
            async move {
                let left_context = get_context_response(&moved_client, left).await;
                let right_context = get_context_response(&moved_client, right).await;

                (left_context, right_context)
            }
        })
    });

    Ok(())
}

fn print_content(meta_data: (MetaData, MetaData)) {
    let inline_change = diff_lines(&meta_data.0.url, &meta_data.1.url)
        .set_show_lines(true)
        .names(&meta_data.0.name, &meta_data.1.name)
        .prettytable();

    println!("{inline_change:?}");
}

async fn get_context_response(client: &Client, data: MetaData) -> Result<ContextResponse> {
    let response = client.get(&data.url).send().await?;
    let status_code = response.status();
    let text = response.text().await?;
    Ok(ContextResponse::new(data.name, status_code, text))
}
