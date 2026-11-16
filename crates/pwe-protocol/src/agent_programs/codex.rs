use pwe_domain::{AgentState, ProgramName, ProgramReport};
use serde_json::Value;

pub fn codex_session(file: &str) -> Option<ProgramReport> {
    let records: Vec<Value> = file
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();
    let header = records
        .iter()
        .find(|record| text(record, "type") == Some("session_meta"))?
        .get("payload")?;
    text(header, "source")?;
    Some(ProgramReport {
        program: ProgramName::Codex,
        session_id: text(header, "id")?.to_string(),
        name: None,
        folder: text(header, "cwd").map(str::to_string),
        account: None,
        pid: None,
        state: records
            .iter()
            .rev()
            .find_map(state_after)
            .unwrap_or(AgentState::Idle),
        waiting_for: None,
        state_since: None,
    })
}

fn state_after(record: &Value) -> Option<AgentState> {
    if text(record, "type") != Some("event_msg") {
        return None;
    }
    match text(record.get("payload")?, "type")? {
        "task_started" => Some(AgentState::Working),
        "task_complete" | "turn_aborted" => Some(AgentState::Idle),
        _ => None,
    }
}

fn text<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

#[cfg(test)]
mod tests;
