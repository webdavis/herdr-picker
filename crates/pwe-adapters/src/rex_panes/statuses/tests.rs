use std::path::PathBuf;

use pwe_application::PaneStatuses;
use pwe_domain::{AgentState, Pane, PaneStatus, ProgramName, WaitingFor, WaitingKind};

use crate::RexPanes;
use crate::cli_command::{RecordedCli, spawning};

const SESSION: &str = "session:00000000-0000-4000-8000-00000000000a";
const SESSIONS: &str = r#"{"sessions":[{"session_id":"session:00000000-0000-4000-8000-00000000000a","label":"invented"}]}"#;
const BLOCKS: &str = r#"{"blocks":[{"block_id":"block:00000000-0000-4000-8000-0000000000b1","label":"shell","creator_name":"com.superlogical.terminal","flavor":"com.superlogical.terminal.shell","host_id":"00000000-0000-4000-8000-0000000000c1","placement":"placed","window_id":"window:00000000-0000-4000-8000-0000000000d1","owner":"client:00000000-0000-4000-8000-0000000000e1"},{"block_id":"block:00000000-0000-4000-8000-0000000000b2","label":"shell","creator_name":"com.superlogical.terminal","flavor":"com.superlogical.terminal.shell","host_id":"00000000-0000-4000-8000-0000000000c1","placement":"placed","window_id":"window:00000000-0000-4000-8000-0000000000d1"}]}"#;
const CLAUDE_BLOCKED: &str = r#"{"records":[{"state":"blocked","kind":"permission","app":"claude-code","effective_app":"claude-code","msg":"approve Bash: sh -c 'echo invented'","updated_at":"2030-01-01T00:00:00.000000-00:00","owner":{"pid":4242,"name":"claude"}}]}"#;
const NO_RECORDS: &str = r#"{"records":[]}"#;
const CLAUDE_IN_FRONT: &str = r#"{"last_exit":null,"child":{"pid":4240,"user_id":501,"user":"someone","name":"login","path":"/usr/bin/login"},"foreground":{"pid":4242,"user_id":501,"user":"someone","name":"claude","path":"/opt/bin/claude","cwd":"/work/project"},"home":"/home/someone"}"#;

fn rex_with_one_blocked_claude() -> RecordedCli {
    RecordedCli::rex(&[
        ("--autostart=false ls --json", SESSIONS),
        (
            "--autostart=false block ls -s session:00000000-0000-4000-8000-00000000000a --json",
            BLOCKS,
        ),
        (
            "--autostart=false block call -b block:00000000-0000-4000-8000-0000000000b1 program_status",
            CLAUDE_BLOCKED,
        ),
        (
            "--autostart=false block call -b block:00000000-0000-4000-8000-0000000000b1 process",
            CLAUDE_IN_FRONT,
        ),
        (
            "--autostart=false block call -b block:00000000-0000-4000-8000-0000000000b2 program_status",
            NO_RECORDS,
        ),
    ])
}

#[test]
fn a_blocked_claude_pane_reads_as_waiting_for_permission() {
    let _spawning = spawning();
    let fake = rex_with_one_blocked_claude();

    let statuses = RexPanes::at(PathBuf::from(&fake.binary), None)
        .pane_statuses()
        .expect("statuses");

    assert_eq!(
        statuses,
        vec![PaneStatus {
            pane: Pane {
                rex_session: SESSION.to_string(),
                tab: "window:00000000-0000-4000-8000-0000000000d1".to_string(),
                block: "block:00000000-0000-4000-8000-0000000000b1".to_string(),
            },
            program: Some(ProgramName::ClaudeCode),
            pid: Some(4242),
            folder: Some("/work/project".to_string()),
            state: AgentState::Waiting,
            waiting_for: Some(WaitingFor {
                kind: WaitingKind::Permission,
                text: "approve Bash: sh -c 'echo invented'".to_string(),
            }),
        }]
    );
}
