#![allow(clippy::doc_markdown, clippy::struct_excessive_bools)]
use anyhow::{Context, Error};
use clap::Parser;
use std::path::PathBuf;

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
    ///  ignore: "regex to ignore lines that matches",{n}
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

        if args.skip_ignore {
            meta_data = meta_data
                .into_iter()
                .map(RequestsConfig::without_ignores)
                .collect();
        }

        if args.all_cache {
            meta_data = meta_data
                .into_iter()
                .map(RequestsConfig::with_cache)
                .collect();
        } else if args.no_cache {
            meta_data = meta_data
                .into_iter()
                .map(RequestsConfig::without_cache)
                .collect();
        }

        Ok(meta_data)
    }
}
