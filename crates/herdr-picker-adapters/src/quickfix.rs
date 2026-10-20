use std::fs;
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

use herdr_picker_domain::QuickfixEntry;
use herdr_picker_protocol::shell_word;
use serde::{Deserialize, Serialize};

use crate::private::write_private;

// The fields Neovim's setqflist() reads.
#[derive(Serialize, Deserialize)]
struct Saved {
    filename: String,
    lnum: u32,
    text: String,
}

pub fn save_quickfix(file: &Path, entries: &[QuickfixEntry]) -> io::Result<()> {
    let saved: Vec<Saved> = entries
        .iter()
        .map(|entry| Saved {
            filename: entry.path.clone(),
            lnum: entry.line,
            text: entry.text.clone(),
        })
        .collect();
    write_private(file, &serde_json::to_vec(&saved)?)
}

pub fn read_quickfix(file: &Path) -> io::Result<Vec<QuickfixEntry>> {
    let saved: Vec<Saved> = serde_json::from_str(&fs::read_to_string(file)?)?;
    Ok(saved
        .into_iter()
        .map(|saved| QuickfixEntry {
            path: saved.filename,
            line: saved.lnum,
            text: saved.text,
        })
        .collect())
}

// Loads the saved list into the Neovim listening on the socket.
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

// A shell command opening Neovim with the saved list loaded, at its numbered entry when given one.
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
