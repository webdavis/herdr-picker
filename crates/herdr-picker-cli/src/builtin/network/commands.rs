use herdr_picker_domain::{Action, PickerCall, PickerRow, Preview};
use herdr_picker_protocol::shell_word;
use serde::Deserialize;

use super::command;
use super::inputs::validate;
use crate::builtin::page::notice;
use crate::builtin::paint::dim;

const CATALOG: &str = include_str!("actions.toml");
// Catalog entries that open another of this picker's sources; any other next is a step.
const SOURCES: [(&str, &str); 3] = [
    ("devices", "devices"),
    ("captures", "captures"),
    ("url", "web checks"),
];

#[derive(Deserialize)]
struct Catalog {
    action: Vec<Entry>,
}

#[derive(Deserialize)]
struct Entry {
    id: String,
    title: String,
    tool: String,
    purpose: String,
    command: Option<String>,
    #[serde(default)]
    params: Vec<String>,
    next: Option<String>,
}

fn catalog() -> Vec<Entry> {
    toml::from_str::<Catalog>(CATALOG).map_or_else(|_| Vec::new(), |catalog| catalog.action)
}

// The network commands menu: Enter prepares a command, after asking for its blanks if it has any.
pub(super) fn command_rows() -> Vec<PickerRow> {
    catalog()
        .into_iter()
        .filter_map(|entry| {
            let label = format!("{}  {}", entry.title, dim(&entry.tool));
            let (Some(text), None) = (&entry.command, &entry.next) else {
                let next = entry.next.as_deref()?;
                let enter = match SOURCES.iter().find(|(name, _)| *name == next) {
                    Some((_, source)) => Action::Source(source.to_string()),
                    None => Action::Next(PickerCall {
                        name: next.to_string(),
                        args: Vec::new(),
                        state: None,
                        query: None,
                    }),
                };
                let detail = format!("{}\nEnter opens the selector.", entry.purpose);
                return Some(PickerRow {
                    label,
                    preview: Some(Preview::Text(detail)),
                    enter,
                    keys: Vec::new(),
                    group: None,
                });
            };
            if entry.params.is_empty() {
                return Some(PickerRow {
                    label,
                    ..command(&entry.title, text, &entry.purpose)
                });
            }
            let asks = format!("Choose {} on Enter.", entry.params.join(", "));
            let detail = [
                entry.title.as_str(),
                text,
                &entry.purpose,
                &asks,
                "Enter prepares the command.",
            ]
            .join("\n");
            let call = PickerCall {
                name: "network-command".into(),
                args: vec![entry.id.clone()],
                state: None,
                query: None,
            };
            Some(PickerRow {
                label,
                preview: Some(Preview::Text(detail)),
                enter: Action::Next(call),
                keys: Vec::new(),
                group: None,
            })
        })
        .collect()
}

// The step where a command's blanks are typed, in order and separated by spaces.
pub(super) fn command_step(id: &str, typed: &str) -> Option<(String, PickerRow)> {
    let entry = catalog().into_iter().find(|entry| entry.id == id)?;
    let text = entry.command.as_deref()?;
    let prompt = format!(
        "Type the {}, separated by spaces, then press Enter.",
        entry.params.join(" and ")
    );
    let row = match filled(text, &entry.params, typed) {
        Ok(filled) => command(&entry.title, &filled, &entry.purpose),
        Err(why) => notice(&why, text),
    };
    Some((prompt, row))
}

fn filled(text: &str, params: &[String], typed: &str) -> Result<String, String> {
    let values: Vec<&str> = typed.split_whitespace().collect();
    if values.len() < params.len() {
        return Err(format!("Type the {}", params[values.len()..].join(" and ")));
    }
    let mut text = text.to_string();
    for (name, value) in params.iter().zip(values) {
        let value = validate(name, value)?;
        text = text.replace(&format!("<{name}>"), &shell_word(&value));
        if name == "cidr" && value.contains(':') {
            text = text.replacen("nmap ", "nmap -6 ", 1);
        }
    }
    Ok(text)
}

#[cfg(test)]
mod tests;
