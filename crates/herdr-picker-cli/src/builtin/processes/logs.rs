use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use herdr_picker_domain::{Action, PickerCall, PickerRow, Preview};
use serde_json::Value;

use crate::builtin::clock::stamp;
use crate::builtin::page::{fields, words};
use crate::builtin::paint::{dim, teal};
use crate::builtin::rows::key;

const REPORTS: [&str; 4] = ["ips", "spin", "crash", "hang"];

// The logs source: two ways in, each its own step.
pub(super) fn log_kinds() -> Vec<PickerRow> {
    let next = |name: &str| {
        Action::Next(PickerCall {
            name: name.to_string(),
            args: Vec::new(),
            state: None,
            query: None,
        })
    };
    let row = |label: &str, detail: &str, step: &str| PickerRow {
        label: label.to_string(),
        preview: Some(Preview::Text(detail.to_string())),
        enter: next(step),
        keys: Vec::new(),
        group: None,
    };
    vec![
        row(
            "Unified logs from the last hour",
            "Read the local unified log. May require additional permissions.",
            "unified-logs",
        ),
        row(
            "Crash and hang reports",
            "Browse accessible DiagnosticReports and prepare opening the original file in Console.",
            "crashes",
        ),
    ]
}

// One line of `log show --style ndjson`: Enter copies its message.
pub(super) fn log_row(line: &str) -> Option<PickerRow> {
    let event: Value = serde_json::from_str(line).ok().filter(Value::is_object)?;
    let said = |field: &str| event[field].as_str().unwrap_or_default().to_string();
    let (message, time, process) = (
        said("eventMessage"),
        said("timestamp"),
        said("processImagePath"),
    );
    let name = Path::new(&process)
        .file_name()
        .map_or(String::new(), |name| name.to_string_lossy().into_owned());
    let details = fields(&[
        ("Time", time.clone()),
        ("Process", process.clone()),
        ("Type", said("eventType")),
        ("Subsystem", said("subsystem")),
    ]);
    let full = serde_json::to_string_pretty(&event).unwrap_or_default();
    Some(PickerRow {
        label: format!("{} {} {message}", dim(&time), teal(&name)),
        preview: Some(Preview::Text(format!(
            "{name}\n\n{details}\n\nFull log entry\n{full}"
        ))),
        enter: Action::Copy(message),
        keys: Vec::new(),
        group: None,
    })
}

// The reports under these folders, newest first: Enter prepares opening one in Console.
pub(super) fn crash_rows(folders: &[PathBuf], offset: i64) -> Vec<PickerRow> {
    let mut reports = Vec::new();
    for folder in folders {
        collect(folder, &mut reports);
    }
    reports.sort_by(|(left, _), (right, _)| right.cmp(left));
    reports
        .into_iter()
        .map(|(modified, path)| {
            let full = path.to_string_lossy().into_owned();
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            let open = words(&["open", "-a", "Console", &full]);
            PickerRow {
                label: format!("{name}  {}", dim(&stamp(modified, offset))),
                preview: Some(Preview::Path(full)),
                enter: Action::Insert(open.clone()),
                keys: vec![(key("ctrl-enter"), Action::Run(open))],
                group: None,
            }
        })
        .collect()
}

fn collect(folder: &Path, reports: &mut Vec<(i64, PathBuf)>) {
    for entry in fs::read_dir(folder).into_iter().flatten().flatten() {
        let path = entry.path();
        let Ok(found) = entry.metadata() else {
            continue;
        };
        if found.is_dir() {
            collect(&path, reports);
        } else if path
            .extension()
            .is_some_and(|suffix| REPORTS.iter().any(|kind| suffix == *kind))
        {
            let modified = found
                .modified()
                .ok()
                .and_then(|at| at.duration_since(UNIX_EPOCH).ok());
            reports.push((modified.map_or(0, |since| since.as_secs() as i64), path));
        }
    }
}

#[cfg(test)]
mod tests;
