use std::fs;

use herdr_picker_domain::{Action, PickerRow};
use serde_json::Value;

use crate::builtin::page::{command_row, words};
use crate::builtin::paint::{dim, green, peach, red};
use crate::builtin::rows::key;

const BREW_FIELDS: [(&str, &str); 5] = [
    ("status", "Status"),
    ("user", "Runs as"),
    ("file", "Service file"),
    ("pid", "PID"),
    ("exit_code", "Last exit code"),
];
// `brew services list --json`: Enter prepares `brew services info`, Alt-D tails its log files.
pub(super) fn brew_rows(listed: &str) -> Result<Vec<PickerRow>, String> {
    let services: Vec<Value> = serde_json::from_str(listed).map_err(|why| why.to_string())?;
    Ok(services
        .iter()
        .map(|service| {
            let name = text(&service["name"]);
            let detail = BREW_FIELDS
                .iter()
                .map(|(field, label)| (label, text(&service[field])))
                .filter(|(_, value)| !matches!(value.as_str(), "" | "none"))
                .map(|(label, value)| format!("{label}: {value}"))
                .collect::<Vec<_>>()
                .join("\n");
            let mut row = service_row(
                &name,
                &text(&service["status"]),
                &["brew", "services", "info", &name],
                &detail,
                "Homebrew",
            );
            let plist = fs::read_to_string(text(&service["file"])).unwrap_or_default();
            let logs = log_paths(&plist);
            if !logs.is_empty() {
                let tail = ["tail", "-n", "200", "--"]
                    .into_iter()
                    .chain(logs.iter().map(String::as_str));
                row.keys.push((
                    key("alt-d"),
                    Action::Insert(words(&tail.collect::<Vec<_>>())),
                ));
            }
            row
        })
        .collect())
}

// The files a launchd plist sends a service's output to, each once.
pub(super) fn log_paths(plist: &str) -> Vec<String> {
    let mut paths: Vec<String> = Vec::new();
    for field in ["StandardOutPath", "StandardErrorPath"] {
        let value = plist
            .split_once(&format!("<key>{field}</key>"))
            .and_then(|(_, after)| after.trim_start().strip_prefix("<string>"))
            .and_then(|after| after.split_once("</string>"))
            .map(|(path, _)| path.to_string());
        paths.extend(value.filter(|path| !paths.contains(path)));
    }
    paths
}

// `launchctl list`: Enter prepares `launchctl list <label>`; Alt-D reads a running job's log.
pub(super) fn launchd_rows(listed: &str) -> Vec<PickerRow> {
    listed
        .lines()
        .skip(1)
        .filter_map(|line| {
            let mut parts = line.splitn(3, char::is_whitespace);
            let (pid, status, label) = (parts.next()?, parts.next()?, parts.next()?.trim());
            let running = pid.parse::<u32>().is_ok();
            let shown = if running {
                format!("pid {pid}")
            } else {
                format!("exit {status}")
            };
            let detail = format!("Label: {label}\nPID: {pid}\nExit status: {status}");
            let mut row = service_row(
                label,
                &shown,
                &["launchctl", "list", label],
                &detail,
                "launchd",
            );
            if running {
                let predicate = format!("processIdentifier == {pid}");
                let log = words(&[
                    "/usr/bin/log",
                    "show",
                    "--last",
                    "1h",
                    "--predicate",
                    &predicate,
                ]);
                row.keys.push((key("alt-d"), Action::Insert(log)));
            }
            Some(row)
        })
        .collect()
}

pub(super) fn service_row(
    name: &str,
    status: &str,
    command: &[&str],
    detail: &str,
    group: &str,
) -> PickerRow {
    PickerRow {
        group: Some(group.to_string()),
        ..command_row(status_line(name, status), words(command), detail)
    }
}

// The name, then the status: green when it runs, red when it failed, dim when there is none.
pub(super) fn status_line(name: &str, status: &str) -> String {
    let lowered = status.to_lowercase();
    let starts = |prefixes: &[&str]| prefixes.iter().any(|prefix| lowered.starts_with(prefix));
    match lowered.as_str() {
        "" => name.to_string(),
        "none" => format!("{name}  {}", dim(status)),
        _ if starts(&["started", "running", "pid", "exit 0"]) => {
            format!("{name}  {}", green(status))
        }
        _ if starts(&["error", "exited", "exit", "dead"]) => format!("{name}  {}", red(status)),
        _ => format!("{name}  {}", peach(status)),
    }
}

pub(super) fn text(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests;
