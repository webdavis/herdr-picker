use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::private::write_private;

// Reading the whole address book takes seconds, so the apple picker keeps a copy, marked with
// when the address book last changed, and reads it while that mark still holds.
pub fn save_contacts(file: &Path, changed: &str, people: &str) -> io::Result<()> {
    write_private(file, format!("{changed}\n{people}").as_bytes())
}

pub fn saved_contacts(file: &Path, changed: &str) -> Option<String> {
    let saved = fs::read_to_string(file).ok()?;
    let (mark, people) = saved.split_once('\n')?;
    (mark == changed).then(|| people.to_string())
}

pub fn contacts_file(state: &Path) -> PathBuf {
    state.join("contacts.json")
}

#[cfg(test)]
mod tests;
