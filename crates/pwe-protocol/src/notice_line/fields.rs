use pwe_domain::{Level, fit_label};
use serde_json::{Map, Value};

use super::NotifyRequest;
use crate::daemon_line::{DaemonError, DaemonErrorKind};

const LEVEL_WORDS: &str = "needs-you, finished, quiet";

pub(super) type Fields = Map<String, Value>;

pub(super) fn notify(fields: &Fields) -> Result<NotifyRequest, DaemonError> {
    let level_word = required(fields, "level")?;
    let level = Level::parse(&level_word).ok_or_else(|| {
        bad_field(format!(
            "\"level\" must be one of {LEVEL_WORDS}, not \"{level_word}\""
        ))
    })?;
    Ok(NotifyRequest {
        source: required(fields, "source")?,
        kind: required(fields, "kind")?,
        key: required(fields, "key")?,
        level,
        text: optional(fields, "text")?.unwrap_or_default(),
        pane: optional(fields, "pane")?,
    })
}

pub(super) fn required(fields: &Fields, name: &str) -> Result<String, DaemonError> {
    optional(fields, name)?
        .filter(|value| !value.is_empty())
        .ok_or_else(|| bad_field(format!("\"{name}\" must be a word")))
}

pub(super) fn optional(fields: &Fields, name: &str) -> Result<Option<String>, DaemonError> {
    match fields.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(fit_label(value))),
        Some(_) => Err(bad_field(format!("\"{name}\" must be text"))),
    }
}

pub(super) fn until(fields: &Fields) -> Result<Option<u64>, DaemonError> {
    match fields.get("until") {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_u64()
            .map(Some)
            .ok_or_else(|| bad_field("\"until\" must be whole seconds since 1970".to_string())),
    }
}

fn bad_field(message: String) -> DaemonError {
    DaemonError {
        kind: DaemonErrorKind::BadField,
        message,
    }
}
