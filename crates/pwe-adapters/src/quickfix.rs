use std::fs;
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

use pwe_domain::QuickfixEntry;
use pwe_protocol::shell_word;
use serde::{Deserialize, Serialize};

use crate::private::write_private_atomically;

#[derive(Serialize, Deserialize)]
struct SetqflistEntry {
    filename: String,
    lnum: u32,
    text: String,
}

pub fn save_quickfix(file: &Path, entries: &[QuickfixEntry]) -> io::Result<()> {
    let saved: Vec<SetqflistEntry> = entries
        .iter()
        .map(|entry| SetqflistEntry {
            filename: entry.path.clone(),
            lnum: entry.line,
            text: entry.text.clone(),
        })
        .collect();
    write_private_atomically(file, &serde_json::to_vec(&saved)?)
}

pub fn read_quickfix(file: &Path) -> io::Result<Vec<QuickfixEntry>> {
    let saved: Vec<SetqflistEntry> = serde_json::from_str(&fs::read_to_string(file)?)?;
    Ok(saved
        .into_iter()
        .map(|saved| QuickfixEntry {
            path: saved.filename,
            line: saved.lnum,
            text: saved.text,
        })
        .collect())
}

pub fn send_quickfix(socket: &str, file: &Path) -> io::Result<()> {
    let ran = Command::new("nvim")
        .args(["--server", socket, "--remote-expr", &load_expression(file)])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .output()?;
    match ran.status.success() {
        true => Ok(()),
        false => Err(io::Error::other(
            String::from_utf8_lossy(&ran.stderr).trim().to_string(),
        )),
    }
}

pub fn open_quickfix(file: &Path, entry: Option<usize>) -> String {
    let load = format!("call {}", load_expression(file));
    let at = entry.map(|number| format!("cc {number}"));
    ["nvim", "-c", &load, "-c", "copen"]
        .into_iter()
        .map(str::to_string)
        .chain(at.into_iter().flat_map(|at| ["-c".to_string(), at]))
        .map(|word| shell_word(&word))
        .collect::<Vec<_>>()
        .join(" ")
}

fn load_expression(file: &Path) -> String {
    let vim_string = file.to_string_lossy().replace('\'', "''");
    format!("setqflist(json_decode(join(readfile('{vim_string}'))))")
}

#[cfg(test)]
mod tests;
