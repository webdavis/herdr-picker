use std::collections::BTreeMap;
use std::path::Path;

use herdr_picker_domain::{Action, PickerRow, Preview};
use herdr_picker_protocol::shell_word;

use super::super::paint::dim;
use super::super::steps::{confirm, next};
use super::files::{absolute, short};
use super::line::{redact, setting_line};
use super::{Entry, SECRET_WORDS};
use crate::builtin::rows::key;

pub(super) fn setting_row(
    entry: &Entry,
    note: &str,
    several: bool,
    managed: &BTreeMap<String, Option<String>>,
    cwd: &Path,
) -> PickerRow {
    let name = redact(&entry.name);
    let secret = SECRET_WORDS
        .iter()
        .any(|word| entry.name.to_lowercase().contains(word));
    let shown = if secret {
        "[redacted]".to_string()
    } else {
        redact(&entry.value)
    };
    let label = match note.is_empty() {
        true => format!("{name} = {}", dim(&shown)),
        false => format!("{}  {}", dim(&format!("{name} = {shown}")), dim(note)),
    };
    let mut detail = format!("Value: {shown}\nOrigin: {}", entry.origin);
    if !note.is_empty() {
        detail += &format!("\nStatus: {note}");
    }
    let file = entry
        .origin
        .strip_prefix("file:")
        .map(|file| absolute(cwd, file));
    let source = file
        .as_ref()
        .and_then(|file| managed.get(file).cloned().flatten());
    let mut keys = Vec::new();
    if let Some(file) = &file {
        let opened = source.clone().unwrap_or_else(|| file.clone());
        let line = Some(setting_line(&opened, &entry.name));
        keys.push((
            key("ctrl-enter"),
            super::super::status::edit_at(Path::new(&opened), line),
        ));
    }
    let refusal = match (&file, &source) {
        (None, _) => Some(format!(
            "{name} was set on the command line, not in a file."
        )),
        _ if several => Some(format!(
            "{name} holds several values; edit its file with Ctrl-Enter."
        )),
        (Some(file), Some(source)) => Some(format!(
            "{} is managed by chezmoi, so the next apply would overwrite a change here. Press Ctrl-Enter to edit {} instead.",
            short(file),
            short(source)
        )),
        _ => None,
    };
    match (refusal, &file) {
        (Some(refusal), _) => detail += &format!("\n\n{refusal}"),
        (None, Some(file)) => {
            let arguments = [file.as_str(), &entry.name, &entry.scope, &shown];
            let change = next(
                "change-setting",
                &arguments,
                (!secret).then_some(&*shown),
                None,
            );
            let remove = ["git", "config", "--file", file, "--unset", &entry.name]
                .map(shell_word)
                .join(" ");
            let question = format!("Remove {} = {shown} from {}?", entry.name, entry.scope);
            let remove = confirm(&question, remove);
            keys.extend([(key("ctrl-v"), change), (key("ctrl-x"), remove)]);
        }
        (None, None) => {}
    }
    PickerRow {
        label,
        preview: Some(Preview::Text(detail)),
        enter: Action::Insert(name),
        keys,
        group: Some(entry.scope.clone()),
    }
}
