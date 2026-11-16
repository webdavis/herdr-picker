use serde_json::Value;

use crate::{PaneInfo, ProtocolError};

const AGENTS_BY_APP: [(&str, &str); 2] = [("claude-code", "claude"), ("codex", "codex")];
const AGENTS_BY_PROGRAM: [&str; 2] = ["claude", "codex"];
const NO_STATUS: &str = "unknown";

pub fn rex_pane(target: &str, status: &str, process: &str) -> Result<PaneInfo, ProtocolError> {
    let status = parsed(status, "program_status")?;
    let process = parsed(process, "process")?;
    let record = status.pointer("/records/0");
    let in_front = process
        .get("foreground")
        .filter(|foreground| !foreground.is_null())
        .or_else(|| process.get("child"));
    let program = in_front.and_then(|found| found["name"].as_str());
    let agent_by_app = record
        .and_then(|record| record["app"].as_str())
        .and_then(|app| AGENTS_BY_APP.iter().find(|(named, _)| *named == app))
        .map(|(_, agent)| *agent);
    let agent_by_program = program.filter(|program| AGENTS_BY_PROGRAM.contains(program));
    Ok(PaneInfo {
        pane_id: target.to_string(),
        workspace_id: String::new(),
        tab_id: String::new(),
        cwd: in_front
            .and_then(|found| found["cwd"].as_str())
            .unwrap_or_default()
            .to_string(),
        agent: agent_by_app.or(agent_by_program).map(str::to_string),
        status: record
            .and_then(|record| record["state"].as_str())
            .unwrap_or(NO_STATUS)
            .to_string(),
    })
}

pub fn opened_block(answer: &str) -> Result<String, ProtocolError> {
    parsed(answer, "new block")?
        .pointer("/block_ids/0")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| ProtocolError::Malformed(format!("Rex named no new block in {answer}")))
}

pub(super) fn parsed(text: &str, method: &str) -> Result<Value, ProtocolError> {
    serde_json::from_str(text)
        .map_err(|failure| ProtocolError::Malformed(format!("Rex {method} answer: {failure}")))
}
