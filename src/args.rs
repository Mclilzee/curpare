#![allow(clippy::doc_markdown, clippy::struct_excessive_bools)]
use anyhow::{Context, Error};
use clap::Parser;
use reqwest::Request;
use std::{collections::HashMap, error::request_ref, path::PathBuf};

use crate::client::RequestsConfig;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = "Takes multiple links and compare their results between eachother"
)]
pub struct Args {
    /// Path of the json file format to load for urls configurations. The configuration should be a list of objects each having left, and right.
    /// each object will be formatted in this format
    ///
    ///{{n}
    ///  name:"name of the comparison",{n}
    ///  url: "https://example.com",{n}
    ///  ignore_lines:[] An array of text. If the line contains this text it will be ignored{n}
    ///  json: boolean # if set to true it will format the json,{n}
    ///  cached: boolean # will cache the call to be used again{n}
    ///}
    pub path: PathBuf,

    /// Clear old cache for this json config
    #[arg(short = 'c', long = "clear-cache")]
    pub clear_cache: bool,

    /// Cache all the calls for this json config
    #[arg(short = 'a', long = "all-cache")]
    pub all_cache: bool,

    /// Don't use cache for any calls for this json config
    #[arg(short = 'n', long = "no-cache")]
    pub no_cache: bool,

    /// Skip all the ignore lines
    #[arg(short = 'i', long = "skip-ignore")]
    pub skip_ignore: bool,
}

impl TryFrom<&Args> for Vec<RequestsConfig> {
    type Error = Error;

    fn try_from(args: &Args) -> Result<Self, Self::Error> {
        let json = std::fs::read_to_string(&args.path).expect("Failed to read json file");
        let mut meta_data: Vec<RequestsConfig> =
            serde_json::from_str(&json).with_context(|| {
                format!(
                    "Json in path {} is not formatted correctly",
                    args.path.display()
                )
            })?;

        let envs: HashMap<String, String> = std::env::vars().collect();
        for config in meta_data.iter_mut() {
            if args.skip_ignore {
                config.left.ignore_lines = None;
                config.right.ignore_lines = None;
            }

            if args.all_cache {
                config.left.cached = true;
                config.right.cached = true;
            } else if args.no_cache {
                config.left.cached = false;
                config.right.cached = false;
            }

            process_env_variables(config, &envs);
        }

        Ok(meta_data)
    }
}

fn process_env_variables(config: &mut RequestsConfig, envs: &HashMap<String, String>) {
    if config.left.url.contains("{}");
}
