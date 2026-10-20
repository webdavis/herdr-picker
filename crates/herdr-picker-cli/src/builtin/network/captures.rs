use std::fs;
use std::path::Path;

use herdr_picker_domain::{Action, PickerCall, PickerRow, Preview};
use serde_json::{Value, json};

use crate::builtin::page::notice;
use crate::builtin::paint::dim;
use crate::builtin::rows::key;

const SKIPPED: [&str; 4] = ["node_modules", "target", ".build", ".git"];
const SUFFIXES: [&str; 3] = ["pcap", "pcapng", "cap"];

// Alt-. shows hidden captures and Ctrl-T sorts newest first, kept in the state; Alt-F picks the folder.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct Shown {
    pub(super) hidden: bool,
    pub(super) newest_first: bool,
}

impl Shown {
    pub(super) fn from_state(state: Option<&str>) -> Self {
        let read: Value = state
            .and_then(|state| serde_json::from_str(state).ok())
            .unwrap_or_default();
        Self {
            hidden: read["hidden"] == true,
            newest_first: read["sort"] == true,
        }
    }

    fn shown(self) -> Action {
        Action::State(json!({"hidden": self.hidden, "sort": self.newest_first}).to_string())
    }
}

// The saved packet captures under this folder; Enter chooses how to read one.
pub(super) fn captures(root: &Path, shown: Shown) -> Vec<PickerRow> {
    let mut found = Vec::new();
    collect(root, shown.hidden, &mut found);
    match shown.newest_first {
        true => found.sort_by(|(left, at, _), (right, then, _)| then.cmp(at).then(left.cmp(right))),
        false => found.sort(),
    }
    let (hidden, newest_first) = (!shown.hidden, !shown.newest_first);
    let folder = PickerCall {
        name: "capture-folder".into(),
        args: Vec::new(),
        state: None,
        query: None,
    };
    let toggles = vec![
        (
            key("alt-.").labelled("toggle hidden captures"),
            Shown { hidden, ..shown }.shown(),
        ),
        (
            key("ctrl-t").labelled("toggle newest first"),
            Shown {
                newest_first,
                ..shown
            }
            .shown(),
        ),
        (
            key("alt-f").labelled("captures in another folder"),
            Action::Next(folder),
        ),
    ];
    if found.is_empty() {
        let detail = "Saved .pcap, .pcapng and .cap files under this folder show here.";
        return vec![PickerRow {
            keys: toggles,
            ..notice("No captures under this folder", detail)
        }];
    }
    found
        .into_iter()
        .map(|(path, _, size)| {
            let full = path.to_string_lossy().into_owned();
            let shown_path = path.strip_prefix(root).unwrap_or(&path).to_string_lossy().into_owned();
            let size = grouped(size);
            PickerRow {
                label: format!("{shown_path}  {}", dim(&format!("{size} bytes"))),
                preview: Some(Preview::Text(format!("Path: {full}\nSize: {size} bytes\nEnter chooses summary, packets, conversations or protocols.\nNo capture analysis runs while hovering."))),
                enter: next("capture-analysis", &full),
                keys: toggles.clone(),
                group: None,
            }
        })
        .collect()
}

fn collect(
    folder: &Path,
    hidden: bool,
    found: &mut Vec<(std::path::PathBuf, std::time::SystemTime, u64)>,
) {
    for entry in fs::read_dir(folder).into_iter().flatten().flatten() {
        let (path, name) = (
            entry.path(),
            entry.file_name().to_string_lossy().into_owned(),
        );
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if !hidden && name.starts_with('.') {
            continue;
        }
        if metadata.is_dir() && !SKIPPED.contains(&name.as_str()) {
            collect(&path, hidden, found);
        } else if path.extension().is_some_and(|suffix| {
            SUFFIXES.contains(&suffix.to_string_lossy().to_lowercase().as_str())
        }) {
            found.push((
                path,
                metadata.modified().unwrap_or(std::time::UNIX_EPOCH),
                metadata.len(),
            ));
        }
    }
}

pub(super) fn next(step: &str, path: &str) -> Action {
    Action::Next(PickerCall {
        name: step.to_string(),
        args: vec![path.to_string()],
        state: None,
        query: None,
    })
}

fn grouped(number: u64) -> String {
    let digits = number.to_string();
    let mut shown = String::new();
    for (at, digit) in digits.chars().enumerate() {
        if at > 0 && (digits.len() - at).is_multiple_of(3) {
            shown.push(',');
        }
        shown.push(digit);
    }
    shown
}

#[cfg(test)]
mod tests;
