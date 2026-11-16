use pwe_domain::{AgentState, Pane, PaneStatus, ProgramName, WaitingFor, WaitingKind};
use serde_json::Value;

use super::pane::parsed;
use crate::rex_event::program_status;
use crate::{ProgramStatus, ProtocolError};

const TERMINAL_FLAVOR: &str = "com.superlogical.terminal.shell";

pub fn rex_session_ids(answer: &str) -> Result<Vec<String>, ProtocolError> {
    Ok(listed(&parsed(answer, "ls")?, "sessions")
        .filter_map(|session| text(session, "session_id"))
        .collect())
}

pub fn rex_terminal_panes(session: &str, answer: &str) -> Result<Vec<Pane>, ProtocolError> {
    Ok(listed(&parsed(answer, "block ls")?, "blocks")
        .filter(|block| block["flavor"] == TERMINAL_FLAVOR && block["placement"] == "placed")
        .filter_map(|block| {
            Some(Pane {
                rex_session: session.to_string(),
                tab: text(block, "window_id")?,
                block: text(block, "block_id")?,
            })
        })
        .collect())
}

pub fn rex_status_record(answer: &str) -> Result<Option<ProgramStatus>, ProtocolError> {
    Ok(parsed(answer, "program_status")?
        .pointer("/records/0")
        .and_then(program_status))
}

pub fn rex_pane_status(
    pane: Pane,
    record: &ProgramStatus,
    process: &str,
) -> Result<PaneStatus, ProtocolError> {
    let process = parsed(process, "process")?;
    let in_front = process
        .get("foreground")
        .filter(|foreground| !foreground.is_null())
        .or_else(|| process.get("child"));
    let state = state_of(record);
    Ok(PaneStatus {
        pane,
        program: ProgramName::parse(&record.program),
        pid: record.pid,
        folder: in_front.and_then(|found| text(found, "cwd")),
        waiting_for: (state == AgentState::Waiting).then(|| waiting_for(record)),
        state,
    })
}

fn state_of(record: &ProgramStatus) -> AgentState {
    match record.state.as_str() {
        "blocked" => AgentState::Waiting,
        "working" => AgentState::Working,
        "done" => AgentState::Finished,
        "idle" => AgentState::Idle,
        "error" => AgentState::Failed,
        _ => AgentState::Unknown,
    }
}

fn waiting_for(record: &ProgramStatus) -> WaitingFor {
    WaitingFor {
        kind: match record.kind.as_deref() {
            Some("permission") => WaitingKind::Permission,
            Some("question") => WaitingKind::Question,
            _ => WaitingKind::Other,
        },
        text: record.message.clone().unwrap_or_default(),
    }
}

fn listed<'a>(answer: &'a Value, key: &str) -> impl Iterator<Item = &'a Value> {
    answer[key].as_array().into_iter().flatten()
}

fn text(value: &Value, key: &str) -> Option<String> {
    value[key].as_str().map(str::to_string)
}

#[cfg(test)]
mod tests;
