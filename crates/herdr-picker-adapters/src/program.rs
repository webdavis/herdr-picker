use std::env;
use std::ffi::OsStr;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use herdr_picker_protocol::{shell_word, split_words};

pub fn on_path(name: &str, path_variable: Option<&OsStr>) -> Option<PathBuf> {
    env::split_paths(path_variable?)
        .map(|folder| folder.join(name))
        .find(|candidate| {
            candidate
                .metadata()
                .is_ok_and(|found| found.is_file() && found.permissions().mode() & 0o111 != 0)
        })
}

// A bare name reads better in history; a program the PATH would not find keeps its full path.
pub fn bare_program(command: &str, path_variable: Option<&OsStr>) -> String {
    let Ok(mut words) = split_words(command) else {
        return command.to_string();
    };
    let Some(first) = words.first_mut() else {
        return command.to_string();
    };
    let program = Path::new(first.as_str());
    let Some(name) = program.file_name().and_then(OsStr::to_str) else {
        return command.to_string();
    };
    if !program.is_absolute() || on_path(name, path_variable).as_deref() != Some(program) {
        return command.to_string();
    }
    *first = name.to_string();
    words
        .iter()
        .map(|word| shell_word(word))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests;
