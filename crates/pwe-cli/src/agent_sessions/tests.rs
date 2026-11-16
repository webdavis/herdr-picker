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
