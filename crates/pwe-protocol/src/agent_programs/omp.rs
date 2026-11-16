use pwe_domain::{AgentState, ProgramName, ProgramReport};
use serde::Deserialize;
use serde_json::Value;

use crate::wire::ProtocolError;

#[derive(Deserialize)]
struct Lease {
    held: Vec<String>,
}

pub fn omp_held_session_ids(printed: &str) -> Result<Vec<String>, ProtocolError> {
    serde_json::from_str::<Lease>(printed)
        .map(|lease| lease.held)
        .map_err(|error| {
            ProtocolError::Malformed(format!("omp session lease printed no held list: {error}"))
        })
}

pub fn omp_session(file: &str) -> Option<ProgramReport> {
    let entries: Vec<Value> = file
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();
    let header = entries
        .iter()
        .find(|entry| kind(entry) == Some("session"))?;
    Some(ProgramReport {
        program: ProgramName::Omp,
        session_id: text(header, "id")?,
        name: entries
            .iter()
            .rev()
            .find(|entry| matches!(kind(entry), Some("title" | "title_change")))
            .and_then(|entry| text(entry, "title")),
        folder: text(header, "cwd"),
        account: None,
        pid: None,
        state: entries
            .iter()
            .rev()
            .find_map(state_after)
            .unwrap_or(AgentState::Idle),
        waiting_for: None,
        state_since: None,
    })
}

fn state_after(entry: &Value) -> Option<AgentState> {
    match kind(entry)? {
        "custom" if text(entry, "customType").as_deref() == Some("session_exit") => {
            Some(AgentState::Ended)
        }
        "message" => {
            let message = entry.get("message")?;
            match (text(message, "role")?.as_str(), text(message, "stopReason")) {
                ("user", _) => Some(AgentState::Working),
                ("assistant", Some(reason)) => Some(after_stop(&reason)),
                _ => None,
            }
        }
        _ => None,
    }
}

fn after_stop(reason: &str) -> AgentState {
    match reason {
        "toolUse" => AgentState::Working,
        "error" => AgentState::Failed,
        "stop" | "aborted" => AgentState::Idle,
        _ => AgentState::Unknown,
    }
}

fn kind(entry: &Value) -> Option<&str> {
    entry.get("type").and_then(Value::as_str)
}

fn text(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

#[cfg(test)]
mod tests;
