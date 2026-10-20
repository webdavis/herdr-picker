use std::path::Path;

use herdr_picker_domain::{Action, Location, PickerRow, Preview, unstyled};
use serde_json::Value;

use crate::builtin::paint::{green, red, teal, yellow};

// What a check printed, a row a line: Cargo's JSON messages and `path:line[:column]: message`
// lines open where they point, and any other line is copied.
pub(super) fn diagnostic_rows(output: &str, cwd: &Path) -> Vec<PickerRow> {
    output
        .lines()
        .map(unstyled)
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| diagnostic(&line, cwd))
        .collect()
}

fn diagnostic(line: &str, cwd: &Path) -> Option<PickerRow> {
    let data: Option<Value> = serde_json::from_str(line).ok().filter(Value::is_object);
    if let Some(data) = data {
        return (data["reason"] == "compiler-message")
            .then(|| compiler_message(&data["message"], line, cwd));
    }
    let row = |label: &str, detail: &str, enter| PickerRow {
        label: label.to_string(),
        preview: Some(Preview::Text(detail.to_string())),
        enter,
        keys: Vec::new(),
        group: None,
    };
    Some(match location(line) {
        Some((path, number, message)) => row(line, message, edit(cwd, path, number)),
        None => row(line, line, Action::Copy(line.to_string())),
    })
}

fn compiler_message(message: &Value, line: &str, cwd: &Path) -> PickerRow {
    let level = message["level"].as_str().unwrap_or_default();
    let text = message["message"].as_str().unwrap_or_default();
    let primary = message["spans"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|span| span["is_primary"] == true);
    let at = primary.and_then(|span| {
        let path = span["file_name"].as_str()?;
        Some(edit(
            cwd,
            path,
            u32::try_from(span["line_start"].as_u64()?).ok()?,
        ))
    });
    let painted = match level {
        "error" => red(level),
        "warning" => yellow(level),
        "help" => green(level),
        _ => teal(level),
    };
    let detail = message["rendered"].as_str().or(message["message"].as_str());
    PickerRow {
        label: format!("{painted}: {text}"),
        preview: Some(Preview::Text(detail.unwrap_or(line).to_string())),
        enter: at.unwrap_or_else(|| Action::Copy(format!("{level}: {text}"))),
        keys: Vec::new(),
        group: None,
    }
}

fn edit(cwd: &Path, path: &str, line: u32) -> Action {
    Action::Edit(vec![Location {
        path: cwd.join(path).to_string_lossy().into_owned(),
        line: Some(line),
    }])
}

// The path, line and message of a `path:line:` or `path:line:column:` line; the path ends at the
// first colon that a line number follows.
fn location(line: &str) -> Option<(&str, u32, &str)> {
    line.match_indices(':').find_map(|(colon, _)| {
        let path = &line[..colon];
        let (number, rest) = digits(&line[colon + 1..])?;
        let rest = rest.strip_prefix(':')?;
        let rest = match digits(rest) {
            Some((_, after)) if after.starts_with(':') => &after[1..],
            _ => rest,
        };
        (!path.is_empty()).then_some((path, number.parse().ok()?, rest.trim_start()))
    })
}

fn digits(text: &str) -> Option<(&str, &str)> {
    let end = text
        .find(|character: char| !character.is_ascii_digit())
        .unwrap_or(text.len());
    (end > 0).then(|| text.split_at(end))
}
