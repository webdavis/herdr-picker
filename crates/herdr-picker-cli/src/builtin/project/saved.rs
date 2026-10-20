use std::path::Path;

use herdr_picker_adapters::{open_quickfix, read_quickfix};
use herdr_picker_domain::{Action, Location, PickerRow, Preview};

use super::notice;
use crate::builtin::Places;
use crate::builtin::rows::{key, shown};

// The list Ctrl-Q saved: Enter reopens Neovim with all of it, at the entry picked.
pub(super) fn saved(places: &Places) -> Vec<PickerRow> {
    let entries = read_quickfix(&places.quickfix).unwrap_or_default();
    if entries.is_empty() {
        let detail = "Mark rows with Tab and press Ctrl-Q in any picker to save them here.";
        return vec![notice("No saved results", detail)];
    }
    entries
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let place = shown(Path::new(&entry.path), &places.cwd);
            let edit = Action::Edit(vec![Location {
                path: entry.path.clone(),
                line: Some(entry.line),
            }]);
            PickerRow {
                label: format!("{place}:{}  {}", entry.line, entry.text),
                preview: Some(Preview::Text(entry.text.clone())),
                enter: Action::Run(open_quickfix(&places.quickfix, Some(index + 1))),
                keys: vec![(key("ctrl-enter"), edit)],
                group: None,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests;
