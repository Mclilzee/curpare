mod args;
mod meta_data;

use anyhow::Result;
use args::Args;
use clap::Parser;
use meta_data::MetaData;
use prettydiff::diff_lines;
use reqwest::{Client, Response};
use serde::Deserialize;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let json = std::fs::read_to_string(args.path).expect("Failed to read json file");
    let meta_data: Vec<MetaData> =
        serde_json::from_str(&json).expect("Json is not formatted correctly");
    let client = reqwest::Client::new();
    let response = get_content(&client, "https://demoqa.com/").await?;
    println!(
        "status code: {}\nbody: {}",
        response.status(),
        response.text().await?
    );
    let first: String = String::from("Hello this is a string to compare\n");
    let second: String = String::from("Hello this is another string\n");
    let inline_change = diff_lines(&first, &second)
        .set_show_lines(true)
        .names("lefsdfsdfsdft", "right")
        .prettytable();

    println!("{inline_change:?}");

    Ok(())
}

async fn get_content(client: &Client, url: &str) -> Result<Response> {
    let response = client.get(url).send().await?;
    Ok(response)
}
