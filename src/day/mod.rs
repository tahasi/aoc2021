use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
    path::Path,
};

use anyhow::{Context, Result};

pub mod eight;
pub mod eleven;
pub mod fifteen;
pub mod five;
pub mod four;
pub mod fourteen;
pub mod nine;
pub mod one;
pub mod seven;
pub mod seventeen;
pub mod six;
pub mod sixteen;
pub mod ten;
pub mod thirteen;
pub mod three;
pub mod twelve;
pub mod two;

fn read_lines(file_path: &Path) -> Result<Vec<String>> {
    let file = File::open(file_path).with_context(|| {
        format!("failed to open file '{}'", file_path.display())
    })?;
    let mut lines = Vec::new();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        lines.push(line.with_context(|| {
            format!("failed to read line from '{}'", file_path.display())
        })?);
    }
    Ok(lines)
}

fn read_all_text(file_path: &Path) -> Result<String> {
    let mut file = File::open(file_path).with_context(|| {
        format!("failed to open file '{}'", file_path.display())
    })?;
    let mut buffer = String::new();
    let _size = file.read_to_string(&mut buffer)?;
    Ok(buffer)
}
