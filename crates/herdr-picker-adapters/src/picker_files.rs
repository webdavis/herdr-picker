use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use herdr_picker_domain::{PickerRow, ViewState};
use herdr_picker_protocol::{encode_view_state, parse_row, parse_view_state};

pub fn picker_view_file(cache: &Path) -> PathBuf {
    cache.with_extension("view")
}

pub fn picker_error_file(cache: &Path) -> PathBuf {
    cache.with_extension("error")
}

pub fn picker_header_file(cache: &Path) -> PathBuf {
    cache.with_extension("header")
}

pub fn picker_said_file(cache: &Path) -> PathBuf {
    cache.with_extension("said")
}

pub fn picker_command_file(cache: &Path) -> PathBuf {
    cache.with_extension("command")
}

pub fn read_step_header(cache: &Path) -> Option<String> {
    fs::read_to_string(picker_header_file(cache)).ok()
}

pub fn cached_rows(cache: &Path) -> io::Result<Vec<PickerRow>> {
    Ok(fs::read_to_string(cache)?
        .lines()
        .enumerate()
        .filter_map(|(index, line)| parse_row(line, index + 1).ok())
        .collect())
}

pub fn read_view(cache: &Path) -> ViewState {
    parse_view_state(&fs::read_to_string(picker_view_file(cache)).unwrap_or_default())
}

pub fn write_view(cache: &Path, state: &ViewState) -> io::Result<()> {
    if let Some(directory) = cache.parent() {
        fs::create_dir_all(directory)?;
    }
    fs::write(picker_view_file(cache), encode_view_state(state))
}

pub fn read_run_error(cache: &Path) -> Option<String> {
    fs::read_to_string(picker_error_file(cache)).ok()
}

pub fn remove_run_files(cache: &Path) {
    for file in [
        cache.to_owned(),
        picker_view_file(cache),
        picker_error_file(cache),
        picker_header_file(cache),
        picker_command_file(cache),
        picker_said_file(cache),
    ] {
        let _ = fs::remove_file(file);
    }
}

#[cfg(test)]
mod tests;
