use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::daemon_line::{DaemonError, DaemonErrorKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Listens {
    ThisMachineOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DaemonStatus {
    pub running: bool,
    pub pid: u32,
    pub socket: String,
    pub listens: Listens,
    pub started_at: u64,
}

pub fn daemon_status_result(status: &DaemonStatus) -> Value {
    json!({
        "running": status.running,
        "pid": status.pid,
        "socket": status.socket,
        "listens": status.listens,
        "started-at": status.started_at,
    })
}

pub fn parse_daemon_status(result: &Value) -> Result<DaemonStatus, DaemonError> {
    DaemonStatus::deserialize(result).map_err(|error| DaemonError {
        kind: DaemonErrorKind::UnreadableReply,
        message: format!("the daemon's status is unreadable: {error}"),
    })
}

#[cfg(test)]
mod tests;
