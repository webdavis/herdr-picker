use std::fs;
use std::path::{Path, PathBuf};

use pwe_application::{AgentProgram, OpenFile, OpenFiles};
use pwe_domain::AgentState;

use super::*;

const THREAD: &str = "33333333-3333-7333-8333-333333333333";

struct Held(Vec<PathBuf>);

impl OpenFiles for Held {
    fn opened_by(&self, command: &str) -> Vec<OpenFile> {
        assert_eq!(command, "codex");
        self.0
            .iter()
            .map(|path| OpenFile {
                pid: 107,
                path: path.clone(),
            })
            .collect()
    }
}

struct CodexFolder(PathBuf);

impl CodexFolder {
    fn new(name: &str) -> Self {
        let folder = std::env::temp_dir().join(format!("pwe-codex-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&folder);
        fs::create_dir_all(folder.join("sessions/2026/01/01")).expect("a sessions folder");
        Self(folder)
    }

    fn rollout(&self, id: &str, source: &str) -> PathBuf {
        let file = self.0.join(format!(
            "sessions/2026/01/01/rollout-2026-01-01T00-00-00-{id}.jsonl"
        ));
        let header = format!(
            r#"{{"type":"session_meta","payload":{{"id":"{id}","cwd":"/work/project","source":{source}}}}}"#
        );
        let turn = r#"{"type":"event_msg","payload":{"type":"task_started"}}"#;
        fs::write(&file, format!("{header}\n{turn}\n")).expect("a rollout file");
        file
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for CodexFolder {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn a_rollout_file_codex_holds_open_is_one_session() {
    let codex = CodexFolder::new("held");
    let thread = codex.rollout(THREAD, r#""cli""#);
    let review = codex.rollout("aaaaaaaa-aaaa", r#"{"subagent":{"other":"guardian"}}"#);
    let held = Held(vec![
        thread.clone(),
        thread,
        review,
        PathBuf::from("/work/project/notes.txt"),
    ]);

    let sessions = CodexSessions::on(codex.path(), held)
        .sessions()
        .expect("files read");

    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].session_id, THREAD);
    assert_eq!(sessions[0].folder.as_deref(), Some("/work/project"));
    assert_eq!(sessions[0].state, AgentState::Working);
    assert_eq!(sessions[0].pid, None);
    assert!(sessions[0].state_since.is_some());
}

#[test]
fn a_rollout_file_nobody_holds_is_not_listed() {
    let codex = CodexFolder::new("closed");
    codex.rollout(THREAD, r#""cli""#);

    let sessions = CodexSessions::on(codex.path(), Held(Vec::new())).sessions();

    assert_eq!(sessions, Ok(Vec::new()));
}
