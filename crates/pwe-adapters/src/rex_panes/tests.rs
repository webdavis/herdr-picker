use std::path::PathBuf;

use pwe_protocol::PaneInfo;

use super::*;
use crate::cli_command::{RecordedCli, spawning};

const CLAUDE_BLOCKED: &str = r#"{"records":[{"state":"blocked","kind":"permission","app":"claude-code","effective_app":"claude-code","owner":{"pid":39910,"name":"claude"}}]}"#;
const NO_RECORDS: &str = r#"{"records":[]}"#;

fn process(name: &str) -> String {
    format!(
        r#"{{"last_exit":null,"child":{{"pid":32070,"name":"bash","cwd":"/work"}},"foreground":{{"pid":39910,"name":"{name}","cwd":"/work/repo"}},"home":"/Users/ada"}}"#
    )
}

fn rex_answering(status: &str, process: &str) -> RecordedCli {
    RecordedCli::rex(&[
        (
            "--autostart=false block call -b block:2 program_status",
            status,
        ),
        ("--autostart=false block call -b block:2 process", process),
        (
            "--autostart=false capture -b block:2 --trim",
            "~/repo\n❯ git sta\n",
        ),
    ])
}

fn panes(fake: &RecordedCli) -> RexPanes {
    RexPanes::at(PathBuf::from(&fake.binary))
}

fn read(status: &str, foreground: &str) -> PaneInfo {
    let _spawning = spawning();
    let fake = rex_answering(status, &process(foreground));
    panes(&fake).pane("block:2").expect("a pane")
}

#[test]
fn a_claude_code_pane_reads_as_claude_and_blocked() {
    assert_eq!(
        read(CLAUDE_BLOCKED, "claude"),
        PaneInfo {
            pane_id: "block:2".to_string(),
            workspace_id: String::new(),
            tab_id: String::new(),
            cwd: "/work/repo".to_string(),
            agent: Some("claude".to_string()),
            status: "blocked".to_string(),
        }
    );
}

#[test]
fn without_a_status_record_the_program_in_front_names_the_agent() {
    assert_eq!(read(NO_RECORDS, "claude").agent.as_deref(), Some("claude"));
    assert_eq!(read(NO_RECORDS, "codex").agent.as_deref(), Some("codex"));
    assert_eq!(read(NO_RECORDS, "bash").agent, None);
    assert_eq!(read(NO_RECORDS, "bash").status, "unknown");
}

#[test]
fn a_codex_record_reads_as_codex_whatever_runs_in_front() {
    let working = r#"{"records":[{"state":"working","app":"codex"}]}"#;

    assert_eq!(read(working, "node").agent.as_deref(), Some("codex"));
}

#[test]
fn the_screen_is_rexs_trimmed_capture_of_the_block() {
    let _spawning = spawning();
    let fake = rex_answering(NO_RECORDS, &process("bash"));

    assert_eq!(
        panes(&fake).screen("block:2").expect("a screen"),
        "~/repo\n❯ git sta\n"
    );
    assert_eq!(
        fake.calls(),
        [["--autostart=false", "capture", "-b", "block:2", "--trim"]]
    );
}

#[test]
fn a_rex_that_fails_says_what_it_was_asked() {
    let _spawning = spawning();
    let fake = RecordedCli::rex(&[]);

    let failure = panes(&fake).screen("block:2").expect_err("a failure");

    assert!(
        failure.to_string().contains("capture -b block:2"),
        "{failure}"
    );
}
