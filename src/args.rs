#![allow(clippy::doc_markdown, clippy::struct_excessive_bools)]
use clap::Parser;
use std::path::PathBuf;

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
