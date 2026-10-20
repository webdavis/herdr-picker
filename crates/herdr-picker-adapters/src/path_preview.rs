use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;
use std::process::Command;

use crate::program::on_path;

const SHOWN_LINES: usize = 200;
const SNIFFED_BYTES: u64 = 8192;

// bat and eza color what they show; without them the path is read here, plain.
pub fn path_preview(path: &Path, path_variable: Option<&OsStr>) -> String {
    let shown = fs::metadata(path).and_then(|found| {
        if found.is_dir() {
            return folder(path, path_variable);
        }
        if is_binary(path)? {
            return Ok(format!("Binary file, {} bytes", found.len()));
        }
        file(path, path_variable)
    });
    shown.unwrap_or_else(|failure| format!("Cannot read {}: {failure}", path.display()))
}

fn folder(path: &Path, path_variable: Option<&OsStr>) -> io::Result<String> {
    if let Some(eza) = on_path("eza", path_variable) {
        return output(
            Command::new(eza)
                .args([
                    "-la",
                    "--color=always",
                    "--group-directories-first",
                    "--no-permissions",
                    "--no-filesize",
                    "--time-style=+%Y-%m-%d %H:%M",
                    "--",
                ])
                .arg(path),
        );
    }
    let mut entries: Vec<(bool, String)> = fs::read_dir(path)?
        .filter_map(Result::ok)
        .map(|entry| {
            let folder = entry.file_type().is_ok_and(|kind| kind.is_dir());
            (!folder, entry.file_name().to_string_lossy().into_owned())
        })
        .collect();
    entries.sort();
    Ok(entries
        .into_iter()
        .map(|(file, name)| {
            if file {
                format!("{name}\n")
            } else {
                format!("{name}/\n")
            }
        })
        .collect())
}

fn file(path: &Path, path_variable: Option<&OsStr>) -> io::Result<String> {
    if let Some(bat) = on_path("bat", path_variable) {
        let mut command = Command::new(bat);
        command.args([
            "--color=always",
            "--style=numbers",
            "--paging=never",
            "--wrap=never",
            &format!("--line-range=:{SHOWN_LINES}"),
        ]);
        if let Some(columns) = std::env::var_os("FZF_PREVIEW_COLUMNS") {
            command.arg(format!("--terminal-width={}", columns.to_string_lossy()));
        }
        return output(command.arg("--").arg(path));
    }
    BufReader::new(File::open(path)?)
        .split(b'\n')
        .take(SHOWN_LINES)
        .map(|line| line.map(|line| String::from_utf8_lossy(&line).into_owned() + "\n"))
        .collect()
}

fn is_binary(path: &Path) -> io::Result<bool> {
    let mut start = Vec::new();
    File::open(path)?
        .take(SNIFFED_BYTES)
        .read_to_end(&mut start)?;
    Ok(start.contains(&0))
}

fn output(command: &mut Command) -> io::Result<String> {
    let ran = command.output()?;
    let shown = if ran.status.success() {
        ran.stdout
    } else {
        ran.stderr
    };
    Ok(String::from_utf8_lossy(&shown).into_owned())
}

#[cfg(test)]
mod tests;
