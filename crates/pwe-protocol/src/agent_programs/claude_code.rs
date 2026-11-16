use pwe_domain::{AgentState, ProgramName, ProgramReport, WaitingFor, WaitingKind};
use serde::Deserialize;

use crate::wire::ProtocolError;

const PERMISSION_PROMPT: &str = "permission prompt";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListedSession {
    session_id: String,
    pid: Option<u32>,
    cwd: Option<String>,
    name: Option<String>,
    status: Option<String>,
    state: Option<String>,
    waiting_for: Option<String>,
}

pub fn claude_code_sessions(printed: &str) -> Result<Vec<ProgramReport>, ProtocolError> {
    let listed: Vec<ListedSession> = serde_json::from_str(printed).map_err(|error| {
        ProtocolError::Malformed(format!(
            "claude agents --json printed no session list: {error}"
        ))
    })?;
    Ok(listed.into_iter().map(report).collect())
}

fn report(session: ListedSession) -> ProgramReport {
    let state = session
        .state
        .as_deref()
        .map(background_state)
        .or_else(|| session.status.as_deref().map(interactive_state))
        .unwrap_or(AgentState::Unknown);
    let waiting_for = (state == AgentState::Waiting)
        .then(|| session.waiting_for.map(reason))
        .flatten();
    ProgramReport {
        program: ProgramName::ClaudeCode,
        session_id: session.session_id,
        name: session.name,
        folder: session.cwd,
        account: None,
        pid: session.pid,
        state,
        waiting_for,
        state_since: None,
    }
}

fn interactive_state(status: &str) -> AgentState {
    match status {
        "busy" => AgentState::Working,
        "waiting" => AgentState::Waiting,
        "idle" => AgentState::Idle,
        _ => AgentState::Unknown,
    }
}

fn background_state(state: &str) -> AgentState {
    match state {
        "working" => AgentState::Working,
        "blocked" => AgentState::Waiting,
        "done" => AgentState::Finished,
        _ => AgentState::Unknown,
    }
}

fn reason(text: String) -> WaitingFor {
    let kind = if text == PERMISSION_PROMPT {
        WaitingKind::Permission
    } else {
        WaitingKind::Other
    };
    WaitingFor { kind, text }
}

#[cfg(test)]
mod tests;
