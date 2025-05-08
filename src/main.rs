mod args;
mod url_data;
use std::net::Shutdown::Write;

use prettydiff::{basic::diff, diff_chars, diff_lines, diff_words, owo_colors::OwoColorize};

use anyhow::Result;
use args::Args;
use clap::Parser;

fn main() -> Result<()> {
    let urls = Args::parse();
    let first: String = String::from("Hello this is a string to compare\n");
    let second: String = String::from("Hello this is another string\n");
    let inline_change = diff_lines(&first, &second)
        .set_show_lines(true)
        .names("lefsdfsdfsdft", "right")
        .prettytable();

    println!("{inline_change:?}");

    Ok(())
}
