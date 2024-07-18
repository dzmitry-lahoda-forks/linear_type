// this is the example from the README.md
//! This example demonstrates how to use the `linear_type` crate to model a linear state machine
//! that reads the content of a file.
use linear_type::Linear;
use std::fs::File;
use std::io::{Read, Result};

// define some newtypes for the states
struct Filename(&'static str);
#[derive(Debug)]
struct ReadonlyFile(File);
#[derive(Debug)]
struct FileContent(String);

// define functions that transition from one state to the next
fn open_file(Filename(name): Filename) -> Result<ReadonlyFile> {
    Ok(ReadonlyFile(File::open(name)?))
}

fn read_text(ReadonlyFile(mut file): ReadonlyFile) -> Result<FileContent> {
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    Ok(FileContent(text))
}

fn main() {
    // Create a linear type and transition through the states
    let file_content = Linear::new(Filename("README.md"))
        .map(open_file)
        .map_ok(read_text)
        .unwrap_ok();

    // destructure the file content
    let FileContent(text) = file_content.into_inner();
    assert!(text.contains("# Example"));
}
