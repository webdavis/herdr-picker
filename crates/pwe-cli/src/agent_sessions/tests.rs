use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

use serde_json::Value;

use crate::cli_command::{RecordedCli, spawning};

use super::*;

const PANELESS: &str = r#"[{"pid":108,"id":"99999999","cwd":"/work/project","kind":"background","sessionId":"99999999-9999-4999-8999-999999999999","name":"background-check","status":"busy","state":"working"}]"#;

struct TempHome(PathBuf);

impl TempHome {
    fn new(name: &str) -> Self {
        let folder =
            std::env::temp_dir().join(format!("pwe-cli-agents-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&folder);
        fs::create_dir_all(&folder).expect("a home folder");
        Self(folder)
    }
}

impl Drop for TempHome {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn surroundings(path_variable: OsString, home: &TempHome) -> Surroundings {
    Surroundings {
        path_variable: Some(path_variable),
        home: home.0.clone(),
        codex_folder: home.0.join(".codex"),
        proc_folder: home.0.join("proc"),
        applications: home.0.join("Applications"),
        rex: pwe_adapters::RexSettings::default(),
        agents: pwe_adapters::AgentSettings::default(),
    }
}

#[test]
fn pwe_agents_lists_a_paneless_claude_session() {
    let _guard = spawning();
    let claude = RecordedCli::printing("claude", PANELESS);
    let home = TempHome::new("paneless");
    let path = std::env::join_paths([claude.folder()]).expect("a PATH");

    let (printed, refusals) = listing(&surroundings(path, &home), 1790000100).expect("listed");

    let value: Value = serde_json::from_str(&printed).expect("JSON");
    let session = &value["agent-sessions"][0];
    assert_eq!(session["program"], "claude-code");
    assert_eq!(session["name"], "background-check");
    assert_eq!(session["state"], "working");
    assert_eq!(session["state-from"], "program");
    assert_eq!(session["state-since"], 1790000100);
    assert_eq!(session.get("pane"), None);
    assert!(refusals.is_empty());
}

#[test]
fn with_no_agent_program_present_the_list_is_empty() {
    let home = TempHome::new("empty");
    let nothing = home.0.join("bin");

    let (printed, refusals) =
        listing(&surroundings(nothing.into_os_string(), &home), 5).expect("listed");

    assert_eq!(printed, r#"{"agent-sessions":[]}"#);
    assert!(refusals.is_empty());
}

const IN_A_PANE: &str = r#"[{"pid":4242,"cwd":"/work/project","kind":"interactive","sessionId":"00000000-0000-4000-8000-000000000042","name":"in-a-pane","status":"busy"}]"#;
const SESSIONS: &str = r#"{"sessions":[{"session_id":"session:00000000-0000-4000-8000-0000000000a1","label":"invented"}]}"#;
const BLOCKS: &str = r#"{"blocks":[{"block_id":"block:00000000-0000-4000-8000-0000000000b2","label":"shell","flavor":"com.superlogical.terminal.shell","placement":"placed","window_id":"window:00000000-0000-4000-8000-0000000000d3"}]}"#;
const BLOCKED: &str = r#"{"records":[{"state":"blocked","kind":"permission","app":"claude-code","msg":"approve Bash: true","owner":{"pid":4242,"name":"claude"}}]}"#;
const PROCESS: &str = r#"{"foreground":{"pid":4242,"name":"claude","cwd":"/work/project"}}"#;

#[test]
fn pwe_agents_takes_the_state_from_rex_and_names_the_pane() {
    let _guard = spawning();
    let claude = RecordedCli::printing("claude", IN_A_PANE);
    let rex = RecordedCli::rex(&[
        ("--autostart=false ls --json", SESSIONS),
        (
            "--autostart=false block ls -s session:00000000-0000-4000-8000-0000000000a1 --json",
            BLOCKS,
        ),
        (
            "--autostart=false block call -b block:00000000-0000-4000-8000-0000000000b2 program_status",
            BLOCKED,
        ),
        (
            "--autostart=false block call -b block:00000000-0000-4000-8000-0000000000b2 process",
            PROCESS,
        ),
    ]);
    let home = TempHome::new("in-a-pane");
    let path = std::env::join_paths([claude.folder(), rex.folder()]).expect("a PATH");

    let (printed, refusals) = listing(&surroundings(path, &home), 1790000100).expect("listed");

    let value: Value = serde_json::from_str(&printed).expect("JSON");
    let session = &value["agent-sessions"][0];
    assert_eq!(session["state"], "waiting");
    assert_eq!(session["state-from"], "rex");
    assert_eq!(session["waiting-for"]["kind"], "permission");
    assert_eq!(
        session["pane"],
        serde_json::json!({"rex-session":"session:00000000-0000-4000-8000-0000000000a1","tab":"window:00000000-0000-4000-8000-0000000000d3","block":"block:00000000-0000-4000-8000-0000000000b2"})
    );
    assert!(refusals.is_empty());
}
