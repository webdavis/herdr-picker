mod row;

use std::fs;
use std::io;
use std::path::Path;

use pwe_domain::{InboxLimits, Notice, prune_inbox};

use crate::private::write_private_atomically;
use row::InboxRow;

pub fn read_inbox(file: &Path) -> io::Result<Vec<Notice>> {
    let text = match fs::read_to_string(file) {
        Ok(text) => text,
        Err(missing) if missing.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(failure) => return Err(failure),
    };
    text.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| read_row(line).map_err(|failure| broken_line(index + 1, failure)))
        .collect()
}

pub fn save_inbox_row(file: &Path, row: &Notice) -> io::Result<()> {
    let mut rows = read_inbox(file)?;
    match rows.iter_mut().find(|kept| kept.id == row.id) {
        Some(kept) => *kept = row.clone(),
        None => rows.push(row.clone()),
    }
    write_inbox(file, &rows)
}

pub fn prune_inbox_file(file: &Path, now: u64, limits: &InboxLimits) -> io::Result<()> {
    write_inbox(file, &prune_inbox(read_inbox(file)?, now, limits))
}

fn read_row(line: &str) -> io::Result<Notice> {
    serde_json::from_str::<InboxRow>(line)?.try_into()
}

fn broken_line(number: usize, failure: io::Error) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("inbox line {number} is not a row: {failure}"),
    )
}

fn write_inbox(file: &Path, rows: &[Notice]) -> io::Result<()> {
    let mut text = String::new();
    for row in rows {
        text.push_str(&serde_json::to_string(&InboxRow::from(row))?);
        text.push('\n');
    }
    write_private_atomically(file, text.as_bytes())
}

#[cfg(test)]
mod tests;
