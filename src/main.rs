mod args;
mod meta_data;

use std::io::Write;

use anyhow::{Context, Result, bail};
use args::Args;
use clap::Parser;
use meta_data::{ContextResponse, MetaData};
use prettydiff::{diff_lines, diff_slice, diff_words, owo_colors::OwoColorize};
use reqwest::{Client, Response};
use serde::Deserialize;
use serde_json::Value;
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let json = std::fs::read_to_string(args.path).expect("Failed to read json file");
    let meta_data: Vec<(MetaData, MetaData)> =
        serde_json::from_str(&json).expect("Json is not formatted correctly");

    let client = reqwest::Client::new();
    let mut handles = vec![];
    for (left, right) in meta_data.into_iter() {
        let moved_client = client.clone();
        let handle = tokio::spawn({
            async move {
                let left_context = get_context_response(&moved_client, left).await;
                let right_context = get_context_response(&moved_client, right).await;

                (left_context, right_context)
            }
        });

        handles.push(handle);
    }

    let mut responses = vec![];
    for handle in handles {
        let result = handle.await.expect("Failed to unlock ansync handle");
        match result {
            (Ok(left), Ok(right)) => responses.push((left, right)),
            (Err(e), _) => eprintln!("{e}"),
            (_, Err(e)) => eprintln!("{e}"),
        }
    }

    responses.iter().for_each(print_context);
    Ok(())
}

fn print_context(context: &(ContextResponse, ContextResponse)) {
    println!(
        "{}:{} | {}:{}",
        context.0.name, context.0.url, context.1.name, context.1.url
    );
    let left_json = to_pretty_json(&context.0.text).unwrap_or(context.0.text.clone());
    let right_json = to_pretty_json(&context.1.text).unwrap_or(context.1.text.clone());
    println!("{:?}", diff_lines(&left_json, &right_json).prettytable());
}

async fn get_context_response(client: &Client, data: MetaData) -> Result<ContextResponse> {
    let response = client.get(&data.url).send().await.context(format!(
        "Failed to call url {}, for name {}",
        data.url, data.name
    ))?;

    let status_code = response.status();
    let text = response.text().await.context(format!(
        "Return body from {}, {} is not correct",
        data.url, data.name
    ))?;

    Ok(ContextResponse::new(data.name, data.url, status_code, text))
}

fn to_pretty_json(str: &str) -> Result<String> {
    let json: Value = serde_json::from_str(str)?;
    Ok(serde_json::to_string_pretty(&json)?)
}
