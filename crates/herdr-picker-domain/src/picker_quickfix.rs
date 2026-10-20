use crate::look::unstyled;
use crate::picker_row::{Action, PickerRow};

// One line of Neovim's quickfix list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuickfixEntry {
    pub path: String,
    pub line: u32,
    pub text: String,
}

// A row's place is where its first edit action opens.
pub fn quickfix_entries(rows: &[&PickerRow]) -> Vec<QuickfixEntry> {
    rows.iter()
        .filter_map(|row| {
            let locations = row.offered().find_map(|(_, action)| match action {
                Action::Edit(locations) => Some(locations),
                _ => None,
            })?;
            let text = unstyled(&row.label);
            Some(locations.iter().map(move |location| QuickfixEntry {
                path: location.path.clone(),
                line: location.line.unwrap_or(1),
                text: text.clone(),
            }))
        })
        .flatten()
        .collect()
}

#[cfg(test)]
mod tests;
