use std::fmt;

use herdr_picker_domain::{Action, PickerRow};
use serde::Deserialize;

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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireRow {
    label: String,
    preview: Option<String>,
    enter: WireAction,
    group: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum WireAction {
    Insert(String),
    Copy(String),
    Cd(String),
    Run(String),
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
    Ok(PickerRow {
        label: wire.label,
        preview: wire.preview,
        enter: match wire.enter {
            WireAction::Insert(text) => Action::Insert(text),
            WireAction::Copy(text) => Action::Copy(text),
            WireAction::Cd(path) => Action::Cd(path),
            WireAction::Run(command) => Action::Run(command),
        },
        group: wire
            .group
            .filter(|group| !group.is_empty())
            .map(|group| group.to_lowercase()),
    })
}

pub fn instruction(action: &Action) -> Option<String> {
    match action {
        Action::Insert(text) => Some(format!("insert\t{text}")),
        Action::Cd(path) => Some(format!("cd\t{path}")),
        Action::Run(command) => Some(format!("run\t{command}")),
        Action::Copy(_) => None,
    }
}

pub fn shell_quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', r"'\''"))
}

#[cfg(test)]
mod tests;
