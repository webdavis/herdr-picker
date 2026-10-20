use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use herdr_picker_adapters::{Entry, Walk, walk};

use super::rows::{RowWriter, file_row, named, painted, shown};
use super::{Places, written};
use crate::command::{CommandError, failed};

const RECENT_FOLDERS: [&str; 2] = ["Documents", "Downloads"];

pub(super) fn files(
    source: &str,
    places: &Places,
    rows: &mut RowWriter<impl Write>,
) -> Result<(), CommandError> {
    // Alt-. switches between this folder's files with and without its hidden ones.
    let (root, hidden, depth, toggle) = match source {
        "this folder" => (places.cwd.clone(), false, None, Some("hidden")),
        "hidden" => (places.cwd.clone(), true, None, Some("this folder")),
        "home" => (home(places)?.to_path_buf(), false, Some(2), None),
        "whole disk" => (PathBuf::from("/"), false, Some(3), None),
        "recent" => return recent(places, rows),
        other => {
            return Err(failed(format!(
                "the files picker has no source named {other}"
            )));
        }
    };
    let search = Walk {
        root,
        entry: Entry::Files,
        hidden,
        depth,
    };
    written(walk(&search, places.fd.as_deref(), |path| {
        let label = painted(&shown(path, &places.cwd));
        rows.row(&file_row(path, label, places, toggle))
    }))
}

// Newest first, so every file is read before the first row is shown.
fn recent(places: &Places, rows: &mut RowWriter<impl Write>) -> Result<(), CommandError> {
    let home = home(places)?;
    let mut found: Vec<(SystemTime, PathBuf)> = Vec::new();
    for folder in RECENT_FOLDERS {
        let search = Walk {
            root: home.join(folder),
            entry: Entry::Files,
            hidden: false,
            depth: None,
        };
        written(walk(&search, places.fd.as_deref(), |path| {
            let modified = fs::metadata(path).and_then(|found| found.modified());
            found.push((modified.unwrap_or(UNIX_EPOCH), path.to_path_buf()));
            Ok(())
        }))?;
    }
    found.sort_by(|newer, older| older.cmp(newer));
    written(
        found.iter().try_for_each(|(_, path)| {
            rows.row(&file_row(path, named(path, Some(home)), places, None))
        }),
    )
}

fn home(places: &Places) -> Result<&Path, CommandError> {
    places
        .home
        .as_deref()
        .ok_or_else(|| failed("HOME is unset, so there is no home folder to list"))
}

#[cfg(test)]
mod tests;
