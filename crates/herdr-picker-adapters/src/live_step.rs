use std::fs;
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::AtomicBool;

use herdr_picker_protocol::{PICKER_QUERY_VARIABLE, parse_header, parse_row};
use serde::{Deserialize, Serialize};

use crate::picker_files::{picker_command_file, picker_error_file, picker_header_file};
use crate::picker_run::{PickerRunError, feed::failure};
use crate::private::write_private;

// A step's command, kept beside its rows so fzf can run it again as the query changes.
#[derive(Serialize, Deserialize)]
struct SavedCommand {
    program: String,
    args: Vec<String>,
    env: Vec<(String, Option<String>)>,
}

pub fn save_step_command(cache: &Path, command: &Command) -> io::Result<()> {
    let text = |word: &std::ffi::OsStr| word.to_string_lossy().into_owned();
    let saved = SavedCommand {
        program: text(command.get_program()),
        args: command.get_args().map(text).collect(),
        env: command
            .get_envs()
            .map(|(name, value)| (text(name), value.map(text)))
            .collect(),
    };
    write_private(&picker_command_file(cache), &serde_json::to_vec(&saved)?)
}

// Only a live step keeps its command beside its rows.
pub fn is_live_step(cache: &Path) -> bool {
    picker_command_file(cache).exists()
}

// The rows land whole, so fzf never reads half a list.
pub fn rerun_step(cache: &Path, query: &str) -> Result<(), PickerRunError> {
    let refused = |refusal: PickerRunError| {
        let _ = write_private(&picker_error_file(cache), refusal.to_string().as_bytes());
        Err(refusal)
    };
    let mut command = match saved_command(cache) {
        Some(command) => command,
        None => {
            return refused(PickerRunError::CommandFailed(
                "no command to run".to_string(),
            ));
        }
    };
    let ran = match command
        .env(PICKER_QUERY_VARIABLE, query)
        .stdin(Stdio::null())
        .output()
    {
        Ok(ran) => ran,
        Err(failure) => return refused(PickerRunError::CommandFailed(failure.to_string())),
    };
    let said = String::from_utf8_lossy(&ran.stderr);
    if let Some(refusal) = failure(Ok(ran.status), &AtomicBool::new(false), &said) {
        return refused(refusal);
    }
    let mut rows = String::new();
    for (index, line) in String::from_utf8_lossy(&ran.stdout).lines().enumerate() {
        if let Some(header) = parse_header(line) {
            let _ = write_private(&picker_header_file(cache), header.as_bytes());
        } else if let Err(bad_row) = parse_row(line, index + 1) {
            return refused(PickerRunError::BadRow(bad_row));
        } else {
            rows.push_str(line);
            rows.push('\n');
        }
    }
    let _ = fs::remove_file(picker_error_file(cache));
    write_private(cache, rows.as_bytes()).or_else(|failure| {
        refused(PickerRunError::CannotCache {
            path: cache.to_owned(),
            reason: failure.to_string(),
        })
    })
}

fn saved_command(cache: &Path) -> Option<Command> {
    let saved: SavedCommand =
        serde_json::from_str(&fs::read_to_string(picker_command_file(cache)).ok()?).ok()?;
    let mut command = Command::new(saved.program);
    command.args(saved.args);
    for (name, value) in saved.env {
        match value {
            Some(value) => command.env(name, value),
            None => command.env_remove(name),
        };
    }
    Some(command)
}

#[cfg(test)]
mod tests;
