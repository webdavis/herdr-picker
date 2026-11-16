use std::path::{Path, PathBuf};

use pwe_adapters::{DaemonAskError, ask_daemon};
use pwe_protocol::{DaemonRequest, DaemonStatus, parse_daemon_status};
use serde_json::{Value, json};

#[derive(Debug, PartialEq, Eq)]
pub struct DaemonReport {
    pub status: Option<DaemonStatus>,
    pub enabled: bool,
    pub socket: PathBuf,
}

pub fn daemon_report(socket: &Path, enabled: bool) -> Result<DaemonReport, DaemonAskError> {
    let status = match ask_daemon(socket, DaemonRequest::Status) {
        Ok(result) => Some(parse_daemon_status(&result).map_err(DaemonAskError::Refused)?),
        Err(DaemonAskError::NotRunning) => None,
        Err(error) => return Err(error),
    };
    Ok(DaemonReport {
        status,
        enabled,
        socket: socket.to_path_buf(),
    })
}

impl DaemonReport {
    pub fn json(&self) -> Value {
        let mut report = json!({
            "running": self.status.is_some(),
            "enabled": self.enabled,
            "socket": self.socket.to_string_lossy(),
            "listens": "this-machine-only",
        });
        if let Some(status) = &self.status {
            report["pid"] = json!(status.pid);
            report["started-at"] = json!(status.started_at);
        }
        report
    }

    pub fn plain_words(&self) -> String {
        let running = match &self.status {
            Some(status) => format!("The pwe daemon is running as process {}.", status.pid),
            None => "The pwe daemon is not running.".to_string(),
        };
        let enabled = if self.enabled {
            "It is enabled: it starts at login and comes back after a crash."
        } else {
            "It is not enabled: it stops at the next reboot. `pwe daemon enable` keeps it running."
        };
        format!(
            "{running}\n{enabled}\nIt listens on this machine only, at {}.\n",
            self.socket.display()
        )
    }
}

#[cfg(test)]
mod tests;
