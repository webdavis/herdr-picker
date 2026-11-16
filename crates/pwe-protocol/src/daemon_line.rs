use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonRequest {
    Status,
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DaemonErrorKind {
    UnknownRequest,
    BadField,
    BadLine,
    UnreadableReply,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonError {
    pub kind: DaemonErrorKind,
    pub message: String,
}

const PROTOCOL_FIELD: &str = "pwe-daemon";
const PROTOCOL_VERSION: u64 = 1;

impl DaemonError {
    fn new(kind: DaemonErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl DaemonRequest {
    fn word(self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::Stop => "stop",
        }
    }

    fn named(word: &str) -> Option<Self> {
        [Self::Status, Self::Stop]
            .into_iter()
            .find(|request| request.word() == word)
    }
}

pub fn daemon_request_line(request: DaemonRequest) -> String {
    let line = json!({ PROTOCOL_FIELD: PROTOCOL_VERSION, "request": request.word() });
    format!("{line}\n")
}

pub fn parse_daemon_request(line: &str) -> Result<DaemonRequest, DaemonError> {
    let object: Map<String, Value> = serde_json::from_str(line.trim())
        .map_err(|_| DaemonError::new(DaemonErrorKind::BadLine, "a request is one JSON object"))?;
    if object.get(PROTOCOL_FIELD).and_then(Value::as_u64) != Some(PROTOCOL_VERSION) {
        return Err(DaemonError::new(
            DaemonErrorKind::BadField,
            format!("\"{PROTOCOL_FIELD}\" must be {PROTOCOL_VERSION}"),
        ));
    }
    let word = object
        .get("request")
        .and_then(Value::as_str)
        .ok_or_else(|| DaemonError::new(DaemonErrorKind::BadField, "\"request\" must be a word"))?;
    DaemonRequest::named(word).ok_or_else(|| {
        DaemonError::new(
            DaemonErrorKind::UnknownRequest,
            format!("the daemon does not know the request \"{word}\""),
        )
    })
}

pub fn daemon_reply_line(reply: &Result<Value, DaemonError>) -> String {
    let line = match reply {
        Ok(result) => json!({ "ok": true, "result": result }),
        Err(error) => json!({ "ok": false, "error": error }),
    };
    format!("{line}\n")
}

#[derive(Deserialize)]
struct Reply {
    ok: bool,
    result: Option<Value>,
    error: Option<DaemonError>,
}

pub fn parse_daemon_reply(line: &str) -> Result<Value, DaemonError> {
    let unreadable = || {
        DaemonError::new(
            DaemonErrorKind::UnreadableReply,
            format!(
                "the daemon replied with an unreadable line: {}",
                line.trim()
            ),
        )
    };
    let reply: Reply = serde_json::from_str(line.trim()).map_err(|_| unreadable())?;
    match reply {
        Reply {
            ok: true,
            result: Some(result),
            ..
        } => Ok(result),
        Reply {
            ok: false,
            error: Some(error),
            ..
        } => Err(error),
        _ => Err(unreadable()),
    }
}

#[cfg(test)]
mod tests;
