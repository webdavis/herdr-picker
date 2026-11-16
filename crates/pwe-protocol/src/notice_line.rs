mod fields;

use pwe_domain::Level;
use serde_json::{Map, Value, json};

use crate::daemon_line::{DaemonError, PROTOCOL_FIELD, PROTOCOL_VERSION};
use fields::{Fields, notify, optional, required, until};

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
    Waiting,
    Hide,
    Jump { id: Option<String> },
    Dismiss { id: String },
    Read { id: String },
    Mute { source: String, until: Option<u64> },
    Unmute { source: String },
}

pub fn notice_request_line(request: &NoticeRequest) -> String {
    let (word, fields) = match request {
        NoticeRequest::Notify(notify) => (
            "notify",
            json!({
                "source": notify.source,
                "kind": notify.kind,
                "key": notify.key,
                "level": notify.level.word(),
                "text": notify.text,
                "pane": notify.pane,
            }),
        ),
        NoticeRequest::Resolve { key } => ("resolve", json!({ "key": key })),
        NoticeRequest::Waiting => ("notice-waiting", json!({})),
        NoticeRequest::Hide => ("notice-hide", json!({})),
        NoticeRequest::Jump { id } => ("notice-jump", json!({ "id": id })),
        NoticeRequest::Dismiss { id } => ("notice-dismiss", json!({ "id": id })),
        NoticeRequest::Read { id } => ("notice-read", json!({ "id": id })),
        NoticeRequest::Mute { source, until } => {
            ("mute", json!({ "source": source, "until": until }))
        }
        NoticeRequest::Unmute { source } => ("unmute", json!({ "source": source })),
    };
    let mut line = Map::new();
    line.insert(PROTOCOL_FIELD.to_string(), json!(PROTOCOL_VERSION));
    line.insert("request".to_string(), json!(word));
    if let Value::Object(fields) = fields {
        line.extend(fields.into_iter().filter(|(_, value)| !value.is_null()));
    }
    format!("{}\n", Value::Object(line))
}

pub fn parse_notice_request(line: &str) -> Option<Result<NoticeRequest, DaemonError>> {
    let fields: Fields = serde_json::from_str(line.trim()).ok()?;
    let id = || required(&fields, "id");
    let source = || required(&fields, "source");
    Some(match fields.get("request")?.as_str()? {
        "notify" => notify(&fields).map(NoticeRequest::Notify),
        "resolve" => required(&fields, "key").map(|key| NoticeRequest::Resolve { key }),
        "notice-waiting" => Ok(NoticeRequest::Waiting),
        "notice-hide" => Ok(NoticeRequest::Hide),
        "notice-jump" => optional(&fields, "id").map(|id| NoticeRequest::Jump { id }),
        "notice-dismiss" => id().map(|id| NoticeRequest::Dismiss { id }),
        "notice-read" => id().map(|id| NoticeRequest::Read { id }),
        "mute" => source()
            .and_then(|source| until(&fields).map(|until| NoticeRequest::Mute { source, until })),
        "unmute" => source().map(|source| NoticeRequest::Unmute { source }),
        _ => return None,
    })
}

#[cfg(test)]
mod tests;
