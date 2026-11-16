use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

pub(super) fn entries_of(folder: &Path) -> Vec<PathBuf> {
    fs::read_dir(folder)
        .map(|entries| entries.flatten().map(|entry| entry.path()).collect())
        .unwrap_or_default()
}

pub(super) fn file_name_of(path: &Path) -> Option<&str> {
    path.file_name().and_then(|name| name.to_str())
}

pub(super) fn modified_seconds(path: &Path) -> Option<u64> {
    let modified = fs::metadata(path).ok()?.modified().ok()?;
    modified
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|since| since.as_secs())
}
