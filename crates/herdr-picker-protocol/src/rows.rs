use std::collections::BTreeMap;
use std::fmt;

use herdr_picker_domain::{Action, Key, PickerRow, Preview};
use serde::{Deserialize, Serialize};

mod action;
mod header;
mod wire;

use action::{from_action, into_action};
pub use header::{encode_header, parse_header};
use wire::{WireAction, WirePreview, Words};

// The state a next action hands the picker it opens, as the JSON the action carried.
pub const PICKER_STATE_VARIABLE: &str = "HERDR_PICKER_STATE";
// The source a picker with sources reads its rows from.
pub const PICKER_SOURCE_VARIABLE: &str = "HERDR_PICKER_SOURCE";
// What is typed, for a picker whose rows follow the query.
pub const PICKER_QUERY_VARIABLE: &str = "HERDR_PICKER_QUERY";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowError {
    pub line: usize,
    pub reason: String,
}

impl fmt::Display for RowError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "line {} is not a picker row: {}",
            self.line, self.reason
        )
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct WireRow {
    label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    preview: Option<WirePreview>,
    enter: WireAction,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    keys: BTreeMap<String, WireAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<Words>,
    #[serde(skip_serializing_if = "Option::is_none")]
    group: Option<String>,
}

pub fn parse_row(line: &str, number: usize) -> Result<PickerRow, RowError> {
    let refusal = |reason: String| RowError {
        line: number,
        reason,
    };
    let wire: WireRow =
        serde_json::from_str(line).map_err(|failure| refusal(failure.to_string()))?;
    if wire.label.contains('\n') {
        return Err(refusal("the label spans more than one line".to_string()));
    }
    if wire
        .group
        .as_deref()
        .is_some_and(|group| group.contains(char::is_control))
    {
        return Err(refusal("the group holds a control character".to_string()));
    }
    let value = wire.value.as_ref();
    let mut keys: Vec<(Key, Action)> = Vec::new();
    for (name, action) in wire.keys {
        let key = claimable(&name).map_err(refusal)?;
        if keys.iter().any(|(claimed, _)| *claimed == key) {
            return Err(refusal(format!("{name} is a key the row already claims")));
        }
        keys.push((key, into_action(action, value).map_err(refusal)?));
    }
    Ok(PickerRow {
        label: wire.label,
        preview: wire.preview.map(|preview| match preview {
            WirePreview::Text(text) => Preview::Text(text),
            WirePreview::Path { path } => Preview::Path(path),
            WirePreview::Command { command } => Preview::Command(command),
        }),
        enter: into_action(wire.enter, value).map_err(refusal)?,
        keys,
        group: wire
            .group
            .filter(|group| !group.is_empty())
            .map(|group| group.to_lowercase()),
    })
}

fn claimable(name: &str) -> Result<Key, String> {
    match Key::parse(name) {
        Some(key) if key != Key::enter() => Ok(key),
        Some(_) => Err("enter belongs in the row's enter field".to_string()),
        None => Err(format!("{name} is not a key a row can claim")),
    }
}

pub fn encode_row(row: &PickerRow) -> String {
    let wire = WireRow {
        label: row.label.clone(),
        preview: row.preview.clone().map(|preview| match preview {
            Preview::Text(text) => WirePreview::Text(text),
            Preview::Path(path) => WirePreview::Path { path },
            Preview::Command(command) => WirePreview::Command { command },
        }),
        enter: from_action(&row.enter),
        keys: row
            .keys
            .iter()
            .map(|(key, action)| (key.name().to_string(), from_action(action)))
            .collect(),
        value: None,
        group: row.group.clone(),
    };
    serde_json::to_string(&wire).expect("a row always encodes")
}

pub fn parse_action(text: &str) -> Result<Action, String> {
    let wire: WireAction = serde_json::from_str(text).map_err(|failure| failure.to_string())?;
    into_action(wire, None)
}

pub fn encode_action(action: &Action) -> String {
    serde_json::to_string(&from_action(action)).expect("an action always encodes")
}

#[cfg(test)]
mod tests;
