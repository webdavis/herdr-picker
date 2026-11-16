use std::ffi::OsString;

use pwe_application::AgentProgram;
use pwe_domain::{AgentState, WaitingFor, WaitingKind};

use crate::agent_programs::SystemProcessEnvironment;
use crate::cli_command::{RecordedCli, spawning};

use super::*;

const WAITING: &str = r#"[{"pid":101,"cwd":"/work/project-a","kind":"interactive","startedAt":1790000000000,"sessionId":"22222222-2222-4222-8222-222222222222","name":"session-e","status":"waiting","waitingFor":"permission prompt"}]"#;
const PS_LISTING: &str = "  PID TTY           TIME CMD\n101 ttys001    0:05.12 claude SHELL=/bin/bash CLAUDE_CONFIG_DIR=/home/someone/.claude-work\n";

fn path_of(programs: &[&RecordedCli]) -> OsString {
    std::env::join_paths(programs.iter().map(|program| program.folder())).expect("a PATH")
}

fn sessions_on(path: &OsString) -> Result<Vec<ProgramReport>, AgentReadError> {
    let nowhere = std::env::temp_dir().join("pwe-no-proc-folder");
    let environment = SystemProcessEnvironment::on(Some(path), &nowhere);
    ClaudeCodeSessions::on(Some(path), environment).sessions()
}

#[test]
fn a_waiting_session_reads_its_reason() {
    let _guard = spawning();
    let claude = RecordedCli::printing("claude", WAITING);
    let ps = RecordedCli::printing("ps", PS_LISTING);

    let sessions = sessions_on(&path_of(&[&claude, &ps])).expect("claude answered");

    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].state, AgentState::Waiting);
    assert_eq!(
        sessions[0].waiting_for,
        Some(WaitingFor {
            kind: WaitingKind::Permission,
            text: "permission prompt".to_string(),
        })
    );
    assert_eq!(
        sessions[0].account.as_deref(),
        Some("/home/someone/.claude-work")
    );
    assert_eq!(claude.calls(), vec![vec!["agents", "--json"]]);
    assert_eq!(ps.calls(), vec![vec!["-E", "-ww", "-p", "101"]]);
}

#[test]
fn a_missing_claude_is_skipped() {
    let _guard = spawning();
    let ps = RecordedCli::printing("ps", PS_LISTING);

    assert_eq!(sessions_on(&path_of(&[&ps])), Ok(Vec::new()));
}

#[test]
fn a_claude_that_prints_no_list_is_an_error_naming_the_program() {
    let _guard = spawning();
    let claude = RecordedCli::printing("claude", "not json");

    let refusal = sessions_on(&path_of(&[&claude])).expect_err("unreadable");

    assert_eq!(refusal.program, "claude-code");
}
