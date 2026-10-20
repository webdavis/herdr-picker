use std::collections::BTreeMap;
use std::fmt;

use herdr_picker_domain::{Action, HerdrCall, PickerRow, Preview};
use serde::{Deserialize, Serialize};

mod action;
mod claims;
mod header;
mod wire;

use action::{from_action, from_call, into_action, into_call};
use claims::claimed_keys;
pub use header::{encode_header, parse_header};
use wire::{WireAction, WireHerdr, WirePreview, Words};

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
    // What a claimed key does on this row, in a few words, for help and the footer.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    labels: BTreeMap<String, String>,
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
    let keys = claimed_keys(wire.keys, wire.labels, wire.value.as_ref()).map_err(refusal)?;
    Ok(PickerRow {
        label: wire.label,
        preview: wire.preview.map(|preview| match preview {
            WirePreview::Text(text) => Preview::Text(text),
            WirePreview::Path { path } => Preview::Path(path),
            WirePreview::Command { command } => Preview::Command(command),
        }),
        enter: into_action(wire.enter, wire.value.as_ref()).map_err(refusal)?,
        keys,
        group: wire
            .group
            .filter(|group| !group.is_empty())
            .map(|group| group.to_lowercase()),
    })
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
        labels: row
            .keys
            .iter()
            .filter_map(|(key, _)| Some((key.name().to_string(), key.label()?.to_string())))
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

// One line of JSON, so a herdr call rides in an instruction line whatever text it carries.
pub fn encode_herdr_call(call: &HerdrCall) -> String {
    serde_json::to_string(&from_call(call)).expect("a herdr call always encodes")
}

pub fn parse_herdr_call(text: &str) -> Option<HerdrCall> {
    serde_json::from_str::<WireHerdr>(text).ok().map(into_call)
}

pub fn encode_action(action: &Action) -> String {
    serde_json::to_string(&from_action(action)).expect("an action always encodes")
}

#[cfg(test)]
mod tests;
