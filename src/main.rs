mod args;
mod url_data;

use anyhow::Result;
use args::Args;
use clap::Parser;
use prettydiff::diff_lines;
use reqwest::{Client, Response};

#[tokio::main]
async fn main() -> Result<()> {
    let client = reqwest::Client::new();
    let response = get_content(&client, "https://demoqa.com/").await?;
    println!(
        "status code: {}\nbody: {}",
        response.status().to_string(),
        response.text().await?
    );
    let urls = Args::parse();
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
