use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use herdr_picker_domain::{PickerRow, View};
use herdr_picker_protocol::{encode_view, parse_row, parse_view};

pub fn picker_view_file(cache: &Path) -> PathBuf {
    cache.with_extension("view")
}

pub fn picker_error_file(cache: &Path) -> PathBuf {
    cache.with_extension("error")
}

pub fn cached_rows(cache: &Path) -> io::Result<Vec<PickerRow>> {
    Ok(fs::read_to_string(cache)?
        .lines()
        .enumerate()
        .filter_map(|(index, line)| parse_row(line, index + 1).ok())
        .collect())
}

pub fn read_view(cache: &Path) -> View {
    parse_view(&fs::read_to_string(picker_view_file(cache)).unwrap_or_default())
}

pub fn write_view(cache: &Path, view: &View) -> io::Result<()> {
    fs::write(picker_view_file(cache), encode_view(view))
}

pub fn read_run_error(cache: &Path) -> Option<String> {
    fs::read_to_string(picker_error_file(cache)).ok()
}

pub fn remove_run_files(cache: &Path) {
    for file in [
        cache.to_owned(),
        picker_view_file(cache),
        picker_error_file(cache),
    ] {
        let _ = fs::remove_file(file);
    }
}

#[cfg(test)]
mod tests;
