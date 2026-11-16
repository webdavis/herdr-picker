use pwe_domain::{AgentState, ProgramName, WaitingFor, WaitingKind};

use super::*;

const AGENTS: &str = include_str!("../fixtures/claude-agents.json");

fn read() -> Vec<ProgramReport> {
    claude_code_sessions(AGENTS).expect("the fixture parses")
}

#[test]
fn every_listed_session_is_read_with_its_identity() {
    let sessions = read();

    assert_eq!(sessions.len(), 5);
    let busy = &sessions[2];
    assert_eq!(busy.program, ProgramName::ClaudeCode);
    assert_eq!(busy.session_id, "11111111-1111-4111-8111-111111111111");
    assert_eq!(busy.name.as_deref(), Some("session-a"));
    assert_eq!(busy.folder.as_deref(), Some("/work/project-a"));
    assert_eq!(busy.pid, Some(104));
    assert_eq!(busy.state, AgentState::Working);
    assert_eq!(busy.account, None);
}

#[test]
fn a_waiting_session_carries_its_reason() {
    let waiting = &read()[3];

    assert_eq!(waiting.state, AgentState::Waiting);
    assert_eq!(
        waiting.waiting_for,
        Some(WaitingFor {
            kind: WaitingKind::Permission,
            text: "permission prompt".to_string(),
        })
    );
}

#[test]
fn a_background_session_is_read_from_its_own_state() {
    let sessions = read();

    assert_eq!(sessions[0].state, AgentState::Waiting);
    assert_eq!(sessions[0].pid, None);
    assert_eq!(sessions[4].state, AgentState::Finished);
}

#[test]
fn an_idle_session_reads_as_idle() {
    assert_eq!(read()[1].state, AgentState::Idle);
}

#[test]
fn printed_text_that_is_not_a_list_is_refused() {
    assert!(claude_code_sessions("{\"agents\":1}").is_err());
}
