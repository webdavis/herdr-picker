use pwe_domain::{AgentState, ProgramName};

use super::*;

const SESSION: &str = include_str!("../fixtures/omp-session.jsonl");

fn with_last_line(line: &str) -> String {
    format!("{SESSION}{line}\n")
}

#[test]
fn the_held_ids_are_read_from_the_lease() {
    let printed = r#"{"held":["44444444-4444-7444-8444-444444444444","55555555-5555-7555-8555-555555555555"],"unknown":[]}"#;

    assert_eq!(
        omp_held_session_ids(printed),
        Ok(vec![
            "44444444-4444-7444-8444-444444444444".to_string(),
            "55555555-5555-7555-8555-555555555555".to_string(),
        ])
    );
}

#[test]
fn a_lease_without_held_ids_is_refused() {
    assert!(omp_held_session_ids("[]").is_err());
}

#[test]
fn the_session_file_names_the_session_and_its_folder() {
    let session = omp_session(SESSION).expect("a session header");

    assert_eq!(session.program, ProgramName::Omp);
    assert_eq!(session.session_id, "44444444-4444-7444-8444-444444444444");
    assert_eq!(session.folder.as_deref(), Some("/work/project"));
    assert_eq!(session.name.as_deref(), Some("Greeting check"));
}

#[test]
fn a_tool_call_in_progress_reads_as_working() {
    assert_eq!(
        omp_session(SESSION).map(|session| session.state),
        Some(AgentState::Working)
    );
}

#[test]
fn a_finished_turn_reads_as_idle() {
    let stopped = with_last_line(
        r#"{"type":"message","id":"a9","parentId":"a8","timestamp":"t","message":{"role":"assistant","stopReason":"stop"}}"#,
    );

    assert_eq!(
        omp_session(&stopped).map(|session| session.state),
        Some(AgentState::Idle)
    );
}

#[test]
fn an_error_reads_as_failed_and_an_exit_as_ended() {
    let failed = with_last_line(
        r#"{"type":"message","id":"a9","message":{"role":"assistant","stopReason":"error"}}"#,
    );
    let exited = with_last_line(
        r#"{"type":"custom","customType":"session_exit","id":"a9","data":{"kind":"normal","reason":"dispose"}}"#,
    );

    assert_eq!(
        omp_session(&failed).map(|s| s.state),
        Some(AgentState::Failed)
    );
    assert_eq!(
        omp_session(&exited).map(|s| s.state),
        Some(AgentState::Ended)
    );
}

#[test]
fn a_file_without_a_session_header_is_not_a_session() {
    assert_eq!(
        omp_session("{\"type\":\"title\",\"title\":\"x\"}\nnot json\n"),
        None
    );
}
