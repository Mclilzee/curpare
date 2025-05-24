#![allow(clippy::doc_markdown, clippy::struct_excessive_bools)]
use anyhow::{Context, Error};
use clap::Parser;
use dotenv::dotenv;
use std::{collections::HashMap, path::PathBuf};

use crate::client::{Config, RequestsConfig};

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

impl TryFrom<&Args> for Config {
    type Error = Error;

    fn try_from(args: &Args) -> Result<Self, Self::Error> {
        dotenv().ok();
        let envs: HashMap<String, String> = std::env::vars().collect();
        let json = std::fs::read_to_string(&args.path)
            .map(|json| process_env_variables(&json, &envs))
            .expect("Failed to read json file");

        let mut config: Config = serde_json::from_str(&json).with_context(|| {
            format!(
                "Json in path {} is not formatted correctly",
                args.path.display()
            )
        })?;

        for request_config in &mut config.requests {

            request_config.left.ignore_lines.extend(config.ignore_lines);
            if args.skip_ignore {
                request_config.left.ignore_lines = None;
                request_config.right.ignore_lines = None;
            }

            if args.all_cache {
                request_config.left.cached = true;
                request_config.right.cached = true;
            } else if args.no_cache {
                request_config.left.cached = false;
                request_config.right.cached = false;
            }
        }

        Ok(config)
    }
}

fn process_env_variables(str: &str, envs: &HashMap<String, String>) -> String {
    let mut chars = str.chars();
    let mut replacement: Vec<char> = vec![];
    while let Some(c) = chars.next() {
        if c == '$' {
            if let Some(next_char) = chars.next() {
                if next_char == '{' {
                    let env_variable: String =
                        chars.by_ref().take_while(|&char| char != '}').collect();
                    let new_val: String = envs.get(&env_variable).unwrap_or_else(|| panic!(
                        "Env variable {env_variable}, is not found. Make sure to provide it or add it to `.env`"
                    )).to_string();

                    replacement.extend(new_val.chars());
                } else {
                    replacement.push(c);
                    replacement.push(next_char);
                }
            }
        } else {
            replacement.push(c);
        }
    }

    replacement.iter().collect()
}
