use std::env;

use herdr_picker_adapters::bare_program;
use herdr_picker_domain::{Action, PickerRow, Preview};
use herdr_picker_protocol::shell_word;

use super::paint::dim;
use super::rows::key;

// A row that only says something: its detail is the preview and Enter does nothing.
pub(super) fn notice(label: &str, detail: &str) -> PickerRow {
    PickerRow {
        label: dim(label),
        preview: Some(Preview::Text(detail.to_string())),
        enter: Action::Nothing,
        keys: Vec::new(),
        group: None,
    }
}

// Enter prepares the command at the prompt; Ctrl-Enter runs it now.
pub(super) fn command_row(label: String, command: String, detail: &str) -> PickerRow {
    PickerRow {
        label,
        preview: Some(Preview::Text(format!(
            "{command}\n\n{detail}\nEnter prepares this command."
        ))),
        enter: Action::Insert(command.clone()),
        keys: vec![(key("ctrl-enter"), Action::Run(command))],
        group: None,
    }
}

pub(super) fn words(words: &[&str]) -> String {
    words
        .iter()
        .map(|word| shell_word(word))
        .collect::<Vec<_>>()
        .join(" ")
}

// Named values a line each, the names in one column; empty values are left out.
pub(super) fn fields(fields: &[(&str, String)]) -> String {
    let width = fields.iter().map(|(name, _)| name.len()).max();
    fields
        .iter()
        .filter(|(_, value)| !value.is_empty())
        .map(|(name, value)| format!("{name:<width$}  {value}", width = width.unwrap_or(0)))
        .collect::<Vec<_>>()
        .join("\n")
}

// herdr-picker itself as the shell would run it: by name when it is the one on PATH.
pub(super) fn own_program() -> String {
    env::current_exe().map_or_else(
        |_| "herdr-picker".to_string(),
        |path| {
            bare_program(
                &shell_word(&path.to_string_lossy()),
                env::var_os("PATH").as_deref(),
            )
        },
    )
}
