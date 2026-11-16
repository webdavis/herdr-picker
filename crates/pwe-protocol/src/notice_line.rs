use pwe_domain::{Level, fit_label};
use serde_json::{Map, Value, json};

use crate::daemon_line::{DaemonError, DaemonErrorKind, PROTOCOL_FIELD, PROTOCOL_VERSION};

const LEVEL_WORDS: &str = "needs-you, finished, quiet";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotifyRequest {
    pub source: String,
    pub kind: String,
    pub key: String,
    pub level: Level,
    pub text: String,
    pub pane: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoticeRequest {
    Notify(NotifyRequest),
    Resolve { key: String },
}

type Fields = Map<String, Value>;

pub fn notice_request_line(request: &NoticeRequest) -> String {
    let mut line = json!({ PROTOCOL_FIELD: PROTOCOL_VERSION });
    let fields = match request {
        NoticeRequest::Notify(notify) => json!({
            "request": "notify",
            "source": notify.source,
            "kind": notify.kind,
            "key": notify.key,
            "level": notify.level.word(),
            "text": notify.text,
            "pane": notify.pane,
        }),
        NoticeRequest::Resolve { key } => json!({ "request": "resolve", "key": key }),
    };
    if let (Some(line), Value::Object(fields)) = (line.as_object_mut(), fields) {
        line.extend(fields.into_iter().filter(|(_, value)| !value.is_null()));
    }
    format!("{line}\n")
}

pub fn parse_notice_request(line: &str) -> Option<Result<NoticeRequest, DaemonError>> {
    let fields: Fields = serde_json::from_str(line.trim()).ok()?;
    Some(match fields.get("request")?.as_str()? {
        "notify" => notify(&fields).map(NoticeRequest::Notify),
        "resolve" => required(&fields, "key").map(|key| NoticeRequest::Resolve { key }),
        _ => return None,
    })
}

fn notify(fields: &Fields) -> Result<NotifyRequest, DaemonError> {
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

fn required(fields: &Fields, name: &str) -> Result<String, DaemonError> {
    optional(fields, name)?
        .filter(|value| !value.is_empty())
        .ok_or_else(|| bad_field(format!("\"{name}\" must be a word")))
}

fn optional(fields: &Fields, name: &str) -> Result<Option<String>, DaemonError> {
    match fields.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(fit_label(value))),
        Some(_) => Err(bad_field(format!("\"{name}\" must be text"))),
    }
}

fn bad_field(message: String) -> DaemonError {
    DaemonError {
        kind: DaemonErrorKind::BadField,
        message,
    }
}

#[cfg(test)]
mod tests;
