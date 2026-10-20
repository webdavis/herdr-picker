use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;
use std::thread;

use super::super::output;
use super::Entry;

pub(super) fn absolute(cwd: &Path, name: &str) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let name = match name.strip_prefix("~/") {
        Some(inside) => format!("{home}/{inside}"),
        None => name.to_string(),
    };
    cwd.join(name).to_string_lossy().into_owned()
}

pub(super) fn short(path: &str) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    match path.strip_prefix(&home).filter(|_| !home.is_empty()) {
        Some(inside) => format!("~{inside}"),
        None => path.to_string(),
    }
}

// Each file asked of chezmoi once, all at the same time, since each answer takes a moment.
pub(super) fn chezmoi_sources(entries: &[Entry], cwd: &Path) -> BTreeMap<String, Option<String>> {
    let files: BTreeSet<String> = entries
        .iter()
        .filter_map(|entry| {
            entry
                .origin
                .strip_prefix("file:")
                .map(|file| absolute(cwd, file))
        })
        .collect();
    let asking: Vec<_> = files
        .into_iter()
        .map(|file| {
            let asked = file.clone();
            (file, thread::spawn(move || chezmoi_source(&asked)))
        })
        .collect();
    asking
        .into_iter()
        .map(|(file, answer)| (file, answer.join().ok().flatten()))
        .collect()
}

fn chezmoi_source(path: &str) -> Option<String> {
    let found = output(Command::new("chezmoi").args(["source-path", path])).ok()?;
    let found = String::from_utf8_lossy(&found).trim().to_string();
    (!found.is_empty()).then_some(found)
}
