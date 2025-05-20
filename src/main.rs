#![warn(clippy::all, clippy::pedantic)]

mod args;
mod client;

use std::{
    fmt::Write as FmtWrite,
    fs::remove_file,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
    time::Duration,
};

use anyhow::{Context, Result};
use args::Args;
use bat::PrettyPrinter;
use clap::Parser;
use client::{Client, RequestsConfig, Response};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tempfile::NamedTempFile;
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

    let (terminal_width, _) = term_size::dimensions().unwrap_or((100, 100));
    let text_to_print =
        get_responses(client, configs)
            .await
            .iter()
            .fold(String::new(), |mut output, response| {
                let _ = write!(
                    output,
                    "{}: {} => {}\n{}",
                    response.name,
                    response.left.url,
                    response.right.url,
                    get_delta_result(&response.left.text, &response.right.text, terminal_width)
                );
                output
            });

    PrettyPrinter::new()
        .input_from_bytes(text_to_print.as_bytes())
        .paging_mode(bat::PagingMode::QuitIfOneScreen)
        .print()
        .unwrap();

    Ok(())
}

async fn get_responses(client: Client, meta_data: Vec<RequestsConfig>) -> Vec<Response> {
    let mut handles = vec![];
    let client = Arc::new(Mutex::new(client));
    let progress_bar = ProgressBar::new(meta_data.len() as u64);
    for request in meta_data {
        let moved_client = client.clone();
        let handle =
            tokio::spawn(async move { moved_client.lock().await.get_response(request).await });

        handles.push(handle);
    }

    progress_bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {wide_bar:.cyan/blue} {pos:>7}/{len:7} Sending request for: {msg} ",
        )
        .unwrap(),
    );
    let mut responses = vec![];
    for handle in handles {
        let result = handle.await.expect("Failed to unlock ansync handle");
        progress_bar.inc(1);
        match result {
            Ok(response) => {
                progress_bar.set_message(response.name.to_string());
                responses.push(response);
            }
            Err(e) => eprintln!("{e}"),
        }
    }
    progress_bar.finish();

    responses
}

fn get_cache_location(path: &Path) -> Result<PathBuf> {
    Ok(Path::new("./cache").join(path.file_name().context("Failed to retreive file name")?))
}

fn get_delta_result(left: &str, right: &str, width: usize) -> String {
    let mut left_file = NamedTempFile::new().expect("Failed to create temp file");
    let () = left_file
        .write_all(left.as_bytes())
        .expect("Failed to write to temp file");

    let mut right_file = NamedTempFile::new().expect("Failed to create temp file");
    let () = right_file
        .write_all(right.as_bytes())
        .expect("Failed to write to temp file");

    Command::new("delta")
        .arg(left_file.path())
        .arg(right_file.path())
        .arg("--default-language=json")
        .arg(format!("--width={width}"))
        .arg("--file-style=omit")
        .arg("-s")
        .output()
        .ok()
        .map(|output| output.stdout)
        .and_then(|out| String::from_utf8(out).ok())
        .expect("Failed to run delta, make sure you have `git-delta` from `https://github.com/dandavison/delta` installed")
}
