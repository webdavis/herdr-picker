use pwe_domain::{AgentState, ProgramName};

use super::*;

const ROLLOUT: &str = include_str!("../fixtures/codex-rollout.jsonl");

fn with_last_line(line: &str) -> String {
    format!("{ROLLOUT}{line}\n")
}

#[test]
fn the_rollout_header_names_the_thread_and_its_folder() {
    let thread = codex_session(ROLLOUT).expect("a session header");

    assert_eq!(thread.program, ProgramName::Codex);
    assert_eq!(thread.session_id, "33333333-3333-7333-8333-333333333333");
    assert_eq!(thread.folder.as_deref(), Some("/work/project"));
    assert_eq!(thread.name, None);
    assert_eq!(thread.pid, None);
}

#[test]
fn a_completed_turn_reads_as_idle() {
    assert_eq!(
        codex_session(ROLLOUT).map(|thread| thread.state),
        Some(AgentState::Idle)
    );
}

#[test]
fn a_started_turn_reads_as_working() {
    let started =
        with_last_line(r#"{"type":"event_msg","payload":{"type":"task_started","turn_id":"t2"}}"#);

    assert_eq!(
        codex_session(&started).map(|thread| thread.state),
        Some(AgentState::Working)
    );
}

#[test]
fn an_aborted_turn_reads_as_idle() {
    let aborted = with_last_line(
        r#"{"type":"event_msg","payload":{"type":"task_started","turn_id":"t2"}}
{"type":"event_msg","payload":{"type":"turn_aborted","turn_id":"t2"}}"#,
    );

    assert_eq!(
        codex_session(&aborted).map(|thread| thread.state),
        Some(AgentState::Idle)
    );
}

#[test]
fn a_review_subagent_thread_is_not_a_session() {
    let review = ROLLOUT.replacen(
        r#""source":"cli""#,
        r#""source":{"subagent":{"other":"guardian"}}"#,
        1,
    );

    assert_eq!(codex_session(&review), None);
}

#[test]
fn a_file_without_a_header_is_not_a_session() {
    assert_eq!(codex_session("{\"type\":\"event_msg\"}\n"), None);
}
