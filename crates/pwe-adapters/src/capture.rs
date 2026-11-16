use std::fs;
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};

use crate::private::write_private;

// One run of a check or a test listing, saved for a picker to read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capture {
    pub category: String,
    pub cwd: String,
    pub framework: String,
    pub argv: Vec<String>,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub fn run_capture(
    output: &Path,
    cwd: &Path,
    category: &str,
    framework: &str,
    argv: &[String],
) -> io::Result<Capture> {
    let (program, arguments) = argv
        .split_first()
        .ok_or_else(|| io::Error::other("a capture needs a command"))?;
    let ran = Command::new(program)
        .args(arguments)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .output()?;
    let capture = Capture {
        category: category.to_string(),
        cwd: cwd.to_string_lossy().into_owned(),
        framework: framework.to_string(),
        argv: argv.to_vec(),
        exit_code: ran.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&ran.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&ran.stderr).into_owned(),
    };
    write_private(output, &serde_json::to_vec(&capture)?)?;
    Ok(capture)
}

// The category's saved runs in the folder, by file name; one that cannot be read is skipped.
pub fn read_captures(directory: &Path, category: &str) -> Vec<Capture> {
    let prefix = format!("{category}-");
    let mut files: Vec<_> = fs::read_dir(directory)
        .into_iter()
        .flatten()
        .filter_map(|entry| Some(entry.ok()?.path()))
        .filter(|path| {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            name.starts_with(&prefix) && name.ends_with(".json")
        })
        .collect();
    files.sort();
    files
        .iter()
        .filter_map(|file| serde_json::from_str(&fs::read_to_string(file).ok()?).ok())
        .collect()
}

#[cfg(test)]
mod tests;
