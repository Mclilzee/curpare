mod url_data;

use std::{
    io::Write,
    process::{Command, Stdio},
};

use tempfile::NamedTempFile;

fn main() {
    let first: String = String::from("Hello this is a string to compare\n");
    let second: String = String::from("Hello this is another string\n");

    let mut first_file =
        NamedTempFile::with_prefix("first_file").expect("Failed to create temp file");
    let mut second_file =
        NamedTempFile::with_prefix("second_file").expect("Failed to create temp file");

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
        .ok()
        .map(|output| output.stdout)
        .and_then(|out| String::from_utf8(out).ok())
        .unwrap();

    let mut first_file =
        NamedTempFile::with_prefix("another-first-file").expect("Failed to create temp file");
    let mut second_file =
        NamedTempFile::with_prefix("another_second_file").expect("Failed to create temp file");

    first_file
        .write_all(first.as_bytes())
        .expect("Failed to write to temp file");

    second_file
        .write_all(second.as_bytes())
        .expect("Failed to write to temp file");

    let output2 = Command::new("delta")
        .arg(first_file.path())
        .arg(second_file.path())
        .output()
        .ok()
        .map(|output| output.stdout)
        .and_then(|output| String::from_utf8(output).ok())
        .unwrap();

    different_aproach();
}

fn different_aproach() {
    // Define the two strings you want to compare
    let string1 = "Hello world";
    let string2 = "Hello not world";

    // Create a command to run `delta`
    let mut delta_command = Command::new("delta")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start delta");

    // Get the stdin of the delta command
    {
        let stdin = delta_command.stdin.as_mut().expect("Failed to open stdin");
        // Write the first string to stdin
        writeln!(stdin, "{string1}").expect("Failed to write to stdin");
        // Write the second string to stdin
        writeln!(stdin, "{string2}").expect("Failed to write to stdin");
    }

    // Wait for the command to finish and capture the output
    let output = delta_command
        .wait_with_output()
        .expect("Failed to read stdout");

    // Print the output
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{stdout}");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Error: {stderr}");
    }
}
