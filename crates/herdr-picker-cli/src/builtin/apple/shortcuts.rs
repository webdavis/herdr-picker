use herdr_picker_domain::{Action, PickerRow, Preview};

use crate::builtin::rows::key;

// `shortcuts list --show-identifiers` prints `<name> (<identifier>)` a line each.
pub(super) fn shortcut_rows(listed: &str) -> Vec<PickerRow> {
    listed
        .lines()
        .filter_map(|line| {
            let (name, identifier) = line.strip_suffix(')')?.rsplit_once(" (")?;
            let uuid = identifier.len() == 36
                && identifier
                    .chars()
                    .all(|c| c.is_ascii_hexdigit() || c == '-');
            uuid.then(|| shortcut_row(name, identifier))
        })
        .collect()
}

fn shortcut_row(name: &str, identifier: &str) -> PickerRow {
    let run = format!("shortcuts run {identifier}");
    PickerRow {
        label: name.to_string(),
        preview: Some(Preview::Text(format!(
            "Identifier: {identifier}\nEnter prepares shortcuts run. Ctrl-O opens its editor."
        ))),
        enter: Action::Insert(run.clone()),
        keys: vec![
            (key("ctrl-enter"), Action::Run(run)),
            (
                key("ctrl-o"),
                Action::Insert(format!("shortcuts view {identifier}")),
            ),
        ],
        group: None,
    }
}

#[cfg(test)]
mod tests;
