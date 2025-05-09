mod args;
mod meta_data;

use anyhow::Result;
use args::Args;
use clap::Parser;
use meta_data::{Data, MetaData};
use prettydiff::diff_lines;
use reqwest::{Client, Response};
use serde::Deserialize;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let json = std::fs::read_to_string(args.path).expect("Failed to read json file");
    let meta_data: Vec<MetaData> =
        serde_json::from_str(&json).expect("Json is not formatted correctly");
    meta_data.into_iter().for_each(print_content);

    let client = reqwest::Client::new();
    let response = get_content(&client, "https://demoqa.com/").await?;
    println!(
        "status code: {:#?}\nbody: {:#?}",
        response.status(),
        response.text().await?
    );

    Ok(())
}

async fn get_content(client: &Client, url: &str) -> Result<Response> {}

fn print_content(meta_data: MetaData) {
    let inline_change = diff_lines(&meta_data.left.url, &meta_data.right.url)
        .set_show_lines(true)
        .names("lefsdfsdfsdft", "right")
        .prettytable();

    println!("{inline_change:?}");
}

fn call_url(client: &Client, data: Data) {
    let response = client.get(data.url).send().await?;
    Ok(response)
}
