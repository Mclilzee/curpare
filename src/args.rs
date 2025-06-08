#![allow(clippy::doc_markdown, clippy::struct_excessive_bools)]
use anyhow::{Context, Error};
use clap::Parser;
use dotenv::dotenv;
use std::{collections::HashMap, path::PathBuf};

use crate::client::Config;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = "Takes multiple web links and compare their results between eachother"
)]
pub struct Args {
    /// Path of the json file format to load for urls configurations. The configuration should be map of a list of requests, each request has name, and an object of left and righta list of objects each having left, and right.
    /// each object will be formatted in this format
    ///
    ///+ {n}
    ///+  "ignore_lines": [], An array of strings to be ignored if the line contains them, this ignore lines config is global for all requests{n}
    ///+  "requests": [{n}
    ///+    {{n}
    ///+      "name":"", Mandatory name field, a string holding the name of the comparison{n}
    ///+      "left": {{n}
    ///+         "url": "https://example.com", Mandatory URL field {n}
    ///+         "ignore_lines":[] Optional local ignore_lines{n}
    ///+         "cached": boolean Optional to cache the response and reuse it instead of sending a request again{n}
    ///+         "user": Optional string value if the call requires authentication{n}
    ///+         "password": Optional string value if the call requires authentication{n}
    ///+         "token": Optional token value, if the call requires a token bearer authentication{n}
    ///+      },{n}
    ///+      "right": {}, With the same fields and options as "left"{n}
    ///+    }]{n}
    ///+ }{n}
    ///  Environmental variables can be used, either by providing them on the command level or by including them in a `.env` file. to use them inside the json wrap them in a ${}
    ///  Example: if we have an environmental variable `HOST=https://google.com` and we use `"url": "${HOST}/query` when the program runs it will resolve to `"url": "https://google.com/query`
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
