use std::path::Path;

use herdr_picker_domain::{PickerRow, Preview};

use super::captures::next;
use crate::builtin::page::notice;

// Alt-F's step: the folder typed, or else the current one, opened as its own list of captures.
pub(super) fn folder_rows(typed: &str, cwd: &Path, home: Option<&Path>) -> (String, PickerRow) {
    let prompt = "Type the folder to search for captures, then press Enter.".to_string();
    let typed = typed.trim();
    let folder = match (typed.strip_prefix("~/"), home) {
        (Some(inside), Some(home)) => home.join(inside),
        _ => cwd.join(typed),
    };
    if !folder.is_dir() {
        let row = notice("Capture search directory does not exist", &prompt);
        return (prompt, row);
    }
    let shown = folder.to_string_lossy().trim_end_matches('/').to_string();
    let row = PickerRow {
        label: format!("Search {shown}"),
        preview: Some(Preview::Text(format!(
            "Folder: {shown}\nEnter lists the captures under it."
        ))),
        enter: next("captures-in", &shown),
        keys: Vec::new(),
        group: None,
    };
    (prompt, row)
}

#[cfg(test)]
mod tests;
