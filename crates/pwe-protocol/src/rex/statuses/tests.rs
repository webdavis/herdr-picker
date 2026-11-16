use pwe_domain::{AgentState, Pane, ProgramName, WaitingKind};

use super::*;

const PROCESS: &str = r#"{"child":{"pid":10,"name":"login"},"foreground":{"pid":11,"name":"codex","cwd":"/work/project"}}"#;

fn pane() -> Pane {
    Pane {
        rex_session: "session:00000000-0000-4000-8000-0000000000a1".to_string(),
        tab: "window:00000000-0000-4000-8000-0000000000d1".to_string(),
        block: "block:00000000-0000-4000-8000-0000000000b1".to_string(),
    }
}

fn status(record: &str) -> PaneStatus {
    let answer = format!(r#"{{"records":[{record}]}}"#);
    let record = rex_status_record(&answer).expect("JSON").expect("a record");
    rex_pane_status(pane(), &record, PROCESS).expect("a status")
}

#[test]
fn each_rex_state_reads_as_its_agent_state() {
    let state = |word: &str| status(&format!(r#"{{"state":"{word}","app":"codex"}}"#)).state;

    assert_eq!(state("working"), AgentState::Working);
    assert_eq!(state("done"), AgentState::Finished);
    assert_eq!(state("idle"), AgentState::Idle);
    assert_eq!(state("error"), AgentState::Failed);
    assert_eq!(state("something-new"), AgentState::Unknown);
}

#[test]
fn a_blocked_question_waits_for_an_answer() {
    let waiting = status(r#"{"state":"blocked","kind":"question","msg":"which one?","app":"omp"}"#);

    assert_eq!(waiting.program, Some(ProgramName::Omp));
    let reason = waiting.waiting_for.expect("a reason");
    assert_eq!(reason.kind, WaitingKind::Question);
    assert_eq!(reason.text, "which one?");
}

#[test]
fn the_folder_is_the_program_in_front_and_the_pid_is_the_records_owner() {
    let codex = status(r#"{"state":"idle","app":"codex","owner":{"pid":12,"name":"codex"}}"#);

    assert_eq!(codex.folder.as_deref(), Some("/work/project"));
    assert_eq!(codex.pid, Some(12));
}

#[test]
fn a_pane_with_no_record_has_no_status() {
    assert_eq!(rex_status_record(r#"{"records":[]}"#), Ok(None));
}

#[test]
fn only_placed_terminal_blocks_are_panes() {
    let blocks = r#"{"blocks":[
        {"block_id":"block:00000000-0000-4000-8000-0000000000b1","flavor":"com.superlogical.terminal.shell","placement":"placed","window_id":"window:00000000-0000-4000-8000-0000000000d1"},
        {"block_id":"block:00000000-0000-4000-8000-0000000000b2","flavor":"com.superlogical.terminal.shell","placement":"detached"},
        {"block_id":"block:00000000-0000-4000-8000-0000000000b3","flavor":"com.example.other","placement":"placed","window_id":"window:00000000-0000-4000-8000-0000000000d1"}
    ]}"#;

    let panes =
        rex_terminal_panes("session:00000000-0000-4000-8000-0000000000a1", blocks).expect("JSON");

    assert_eq!(panes, vec![pane()]);
}

#[test]
fn every_session_is_listed_by_its_id() {
    let sessions = r#"{"sessions":[{"session_id":"session:one","label":"a"},{"session_id":"session:two","label":"b"}]}"#;

    assert_eq!(
        rex_session_ids(sessions),
        Ok(vec!["session:one".to_string(), "session:two".to_string()])
    );
}
