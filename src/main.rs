#![warn(clippy::all, clippy::pedantic)]

mod args;
mod client;

use std::{
    fs::{File, remove_file},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};

use anyhow::{Context, Result, anyhow};
use args::Args;
use bat::PrettyPrinter;
use clap::Parser;
use client::{Client, Config, Response};
use indicatif::{ProgressBar, ProgressStyle};
use tempfile::NamedTempFile;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config: Config = (&args).try_into()?;
    let requires_caching = config.requires_cache();
    let cache_location = get_cache_location(&args.path);
    if args.clear_cache {
        let cache_location = cache_location.as_ref().unwrap_or_else(|e| {
            panic!(
                "Failed to handle cache location for path {}: {e:?}",
                args.path.display()
            )
        });

        if cache_location.exists() {
            remove_file(cache_location).with_context(|| {
                format!(
                    "Failed to clear cache for path {}",
                    cache_location.display()
                )
            })?;
        }
    }

    let client = if requires_caching {
        Client::new_cached(cache_location?).context("Failed to load cache")?
    } else {
        Client::new()
    };

    if let Some(path) = args.out {
        save_responses_with_differences(client, config, path).await?;
        return Ok(());
    }

    let responses = get_responses(client, config).await;
    if !args.cache_only {
        print_differences(&responses);
    }
    Ok(())
}

fn print_differences(responses: &[Response]) {
    let (terminal_width, _) = term_size::dimensions().unwrap_or((100, 100));
    let diff = responses
        .iter()
        .map(|response| {
            if response.left.text == response.right.text {
                format!(
                    "{}: {} == {}\n",
                    response.name, response.left.url, response.right.url
                )
            } else {
                format!(
                    "{}: {} => {}\n{}",
                    response.name,
                    response.left.url,
                    response.right.url,
                    get_delta_result(&response.left.text, &response.right.text, terminal_width)
                )
            }
        })
        .collect::<String>();

    PrettyPrinter::new()
        .input_from_bytes(diff.as_bytes())
        .paging_mode(bat::PagingMode::QuitIfOneScreen)
        .print()
        .expect("Failed to show differences using bat");
}

async fn get_responses(client: Client, config: Config) -> Vec<Response> {
    let mut handles = vec![];
    let client = Arc::new(Mutex::new(client));
    let progress_bar = ProgressBar::new(config.requests.len() as u64);
    progress_bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {wide_bar:.cyan/blue} {pos:>7}/{len:7} Sending request for: {msg} ",
        )
        .unwrap(),
    );

    for request in config.requests {
        let moved_client = client.clone();
        let moved_progress_bar = progress_bar.clone();
        let handle = tokio::spawn(async move {
            let result = moved_client.lock().await.get_response(request).await;
            if let Ok(response) = &result {
                moved_progress_bar.set_message(response.name.clone());
            }

            moved_progress_bar.inc(1);
            result
        });

        handles.push(handle);
    }

    let mut responses = vec![];
    for handle in handles {
        let result = handle.await.expect("Failed to unlock ansync handle");
        match result {
            Ok(response) => responses.push(response),
            Err(e) => eprintln!("{e:?}"),
        }
    }

    progress_bar.finish();
    responses
}

async fn save_responses_with_differences(
    client: Client,
    config: Config,
    path: PathBuf,
) -> Result<()> {
    let mut handles = vec![];
    let client = Arc::new(Mutex::new(client));
    let progress_bar = ProgressBar::new(config.requests.len() as u64);
    progress_bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {wide_bar:.cyan/blue} {pos:>7}/{len:7} Sending request for: {msg} ",
        )
        .unwrap(),
    );

    for request in config.requests {
        let moved_client = client.clone();
        let moved_progress_bar = progress_bar.clone();
        let handle = tokio::spawn(async move {
            let result = moved_client
                .lock()
                .await
                .get_response(request.clone())
                .await;
            match &result {
                Ok(response) => {
                    moved_progress_bar.set_message(response.name.clone());
                    moved_progress_bar.inc(1);
                    if response.left.text == response.right.text {
                        Ok(None)
                    } else {
                        Ok(Some(request))
                    }
                }
                Err(e) => {
                    moved_progress_bar.inc(1);
                    Err(anyhow!("{e}"))
                }
            }
        });

        handles.push(handle);
    }

    let mut requests = vec![];
    for handle in handles {
        let result = handle.await.expect("Failed to unlock ansync handle");
        match result {
            Ok(Some(request)) => requests.push(request),
            Ok(None) => {}
            Err(e) => eprintln!("{e:?}"),
        }
    }

    let config = toml::to_string(&Config::from(requests))?;
    let mut file = File::create(path)?;
    file.write(config.as_bytes())?;

    progress_bar.finish();
    Ok(())
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
