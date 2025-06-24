#![allow(clippy::doc_markdown, clippy::struct_excessive_bools)]
use anyhow::{Context, Error};
use clap::Parser;
use dotenv::dotenv;
use std::{collections::HashMap, path::PathBuf};
use toml;

use crate::client::Config;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = "Takes multiple web links and compare their results between eachother"
)]
pub struct Args {
    /// Environmental variables can be used by wrapping them in `${}` within any string value inside the TOML config.
    /// Example: if you have an environment variable `HOST=https://google.com`
    /// and you use `"url": "${HOST}/query"`, when the program runs it will resolve to `"url": "https://google.com/query"`.
    pub path: PathBuf,

    /// Clear old cache for this toml config
    #[arg(short = 'c', long = "clear-cache")]
    pub clear_cache: bool,

    /// Cache all the calls for this toml config
    #[arg(short = 'a', long = "all-cache")]
    pub all_cache: bool,

    /// Don't use cache for any calls for this toml config
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

        let toml = std::fs::read_to_string(&args.path)
            .map(|toml| process_env_variables(&toml, &envs))
            .map_err(|e| Error::msg(format!("Failed to read {}: {}", args.path.display(), e)))?;

        let mut config: Config = toml::from_str(&toml).with_context(|| {
            format!(
                "Toml in path {} is not formatted correctly",
                args.path.display()
            )
        })?;

        for request_config in &mut config.requests {
            if args.skip_ignore {
                request_config.left.ignore_lines = vec![];
                request_config.right.ignore_lines = vec![];
            } else {
                request_config
                    .left
                    .ignore_lines
                    .extend(config.ignore_lines.clone());

                request_config
                    .right
                    .ignore_lines
                    .extend(config.ignore_lines.clone());
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
