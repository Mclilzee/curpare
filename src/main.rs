use std::{
    io::Write,
    process::{Command, Output},
};

use tempfile::NamedTempFile;

fn main() {
    let first: String = String::from("Hello this is a string to compare");
    let second: String = String::from("Hello this is another string");

    let mut first_file = NamedTempFile::new().expect("Failed to create temp file");
    let mut second_file = NamedTempFile::new().expect("Failed to create temp file");

    first_file
        .write_all(first.as_bytes())
        .expect("Failed to write to temp file");

    second_file
        .write_all(second.as_bytes())
        .expect("Failed to write to temp file");

    let output = Command::new("delta")
        .arg(first_file.path())
        .arg(second_file.path())
        .output()
        .unwrap();

    let str = String::from_utf8(output.stdout).unwrap();
    println!("{str}");
}
