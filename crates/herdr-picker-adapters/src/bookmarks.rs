use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub fn read_bookmarks(file: &Path) -> Vec<PathBuf> {
    fs::read_to_string(file)
        .unwrap_or_default()
        .lines()
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .collect()
}

// False when the folder was already bookmarked.
pub fn add_bookmark(file: &Path, folder: &Path) -> io::Result<bool> {
    if read_bookmarks(file).iter().any(|saved| saved == folder) {
        return Ok(false);
    }
    if let Some(directory) = file.parent() {
        fs::create_dir_all(directory)?;
    }
    let mut saved = OpenOptions::new().create(true).append(true).open(file)?;
    writeln!(saved, "{}", folder.display())?;
    Ok(true)
}

#[cfg(test)]
mod tests;
