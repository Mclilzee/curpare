mod args;
mod client;
mod meta_data;

use anyhow::Result;
use args::Args;
use clap::Parser;
use client::Client;
use meta_data::MetaData;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let json = std::fs::read_to_string(args.path).expect("Failed to read json file");
    let meta_data: Vec<(MetaData, MetaData)> =
        serde_json::from_str(&json).expect("Json is not formatted correctly");

    let client = Client::new();
    let mut handles = vec![];
    for (left, right) in meta_data.into_iter() {
        let moved_client = client.clone();
        let handle = tokio::spawn({
            async move {
                println!("Sending requests to {left}, and {right}");
                let left_context = moved_client.get(left).await;
                let right_context = moved_client.get(right).await;

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

    println!("All requests has responded, printing differences");

    responses
        .iter()
        .map(|(left, right)| left.diff(right))
        .for_each(|s| println!("{s}"));
    Ok(())
}
