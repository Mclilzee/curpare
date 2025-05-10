mod args;
mod meta_data;

use anyhow::{Context, Result, bail};
use args::Args;
use clap::Parser;
use meta_data::{ContextResponse, MetaData};
use prettydiff::diff_lines;
use reqwest::{Client, Response};
use serde::Deserialize;
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

    for handle in handles {
        let result = handle.await.expect("Failed to unlock ansync handle");
        match result {
            (Ok(left), Ok(right)) => print_content((left, right)),
            (Err(e), _) => eprintln!("{e}"),
            (_, Err(e)) => eprintln!("{e}"),
        }
    }
    Ok(())
}

fn print_content(context: (ContextResponse, ContextResponse)) {
    println!(
        "{:?}",
        diff_lines(&context.0.text, &context.1.text)
            .set_show_lines(true)
            .names(&context.0.name, &context.1.name)
            .prettytable()
    );
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
