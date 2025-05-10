use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = "Takes multiple links and compare their results between eachother"
)]
pub struct Args {
    /// Path of the json file format to load for urls configurations. The configuration should be a list of objects each having left, and right.
    /// each object will be formatted in this format
    ///{
    ///name:"name of the comparison",
    ///url: "<https://example.com>",
    ///ignore: "regex to ignore lines that matches"
    ///json: boolean, if set to true it will format the json
    /// cached: boolean, will cache the call to be used again
    ///}
    pub path: PathBuf,

    /// Flag to clear the cache before running the program
    #[arg(short = 'c', long = "clear-cache")]
    pub clear_cache: bool,
}
