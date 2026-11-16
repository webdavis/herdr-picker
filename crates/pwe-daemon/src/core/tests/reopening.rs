use super::*;
use std::fs;
use std::path::PathBuf;

use pwe_application::AgentReadError;
use pwe_domain::{AgentState, Pane, PaneStatus, ProgramName, ProgramReport, RecordedPane};

use crate::reopening::Reopening;

type Reopened = Arc<Mutex<Vec<Vec<RecordedPane>>>>;

struct TempFile(PathBuf);

impl TempFile {
    fn new(name: &str) -> Self {
        let folder = std::env::temp_dir().join(format!("pwe-reopen-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&folder);
        Self(folder.join("agent-panes.json"))
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        if let Some(folder) = self.0.parent() {
            let _ = fs::remove_dir_all(folder);
        }
    }
}

fn reopening_core(file: &TempFile) -> (DaemonCore, Reopened) {
    let mut core = core();
    let (asks, _asked) = std::sync::mpsc::sync_channel(1);
    core.watch_agents(AgentWatch::new(asks));
    let reopened = Reopened::default();
    let told = Arc::clone(&reopened);
    core.reopen_after_rex_restarts(Reopening::new(
        file.0.clone(),
        Box::new(move |records| {
            if let Ok(mut reopened) = told.lock() {
                reopened.push(records);
            }
        }),
    ));
    (core, reopened)
}

fn claude_in_a_pane() -> AgentSources {
    AgentSources {
        reports: vec![ProgramReport {
            program: ProgramName::ClaudeCode,
            session_id: "00000000-0000-4000-8000-000000000042".to_string(),
            name: None,
            folder: Some("/work/project".to_string()),
            account: None,
            pid: Some(4101),
            state: AgentState::Idle,
            waiting_for: None,
            state_since: None,
        }],
        panes: vec![PaneStatus {
            pane: pane(),
            program: Some(ProgramName::ClaudeCode),
            pid: Some(4101),
            folder: Some("/work/project".to_string()),
            state: AgentState::Idle,
            waiting_for: None,
        }],
        refusals: Vec::new(),
    }
}

fn pane() -> Pane {
    Pane {
        rex_session: "session:00000000-0000-7000-8000-0000000000a1".to_string(),
        tab: "window:00000000-0000-7000-8000-0000000000c1".to_string(),
        block: "block:00000000-0000-7000-8000-0000000000b1".to_string(),
    }
}

fn rex_gone() -> AgentSources {
    AgentSources {
        refusals: vec![AgentReadError {
            program: "rex".to_string(),
            detail: "the server is not running".to_string(),
        }],
        ..AgentSources::default()
    }
}

fn reopened_blocks(reopened: &Reopened) -> Vec<Vec<String>> {
    reopened
        .lock()
        .map(|reopened| {
            reopened
                .iter()
                .map(|records| {
                    records
                        .iter()
                        .map(|record| record.pane.block.clone())
                        .collect()
                })
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn recorded_panes_are_reopened_once_rex_comes_back() {
    let file = TempFile::new("back");
    let (mut core, reopened) = reopening_core(&file);
    core.hear_rex(RexNews::Linked(RexLink::Connected), 100);
    core.take_agent_reading(claude_in_a_pane(), 101);
    let before_the_restart = reopened_blocks(&reopened);

    core.hear_rex(RexNews::Linked(RexLink::Reconnecting), 200);
    core.take_agent_reading(rex_gone(), 201);
    core.hear_rex(RexNews::Linked(RexLink::Connected), 230);

    assert_eq!(before_the_restart, Vec::<Vec<String>>::new());
    assert_eq!(reopened_blocks(&reopened), vec![vec![pane().block]]);
}

#[test]
fn a_reading_while_rex_is_away_keeps_the_record() {
    let file = TempFile::new("away");
    let (mut core, _reopened) = reopening_core(&file);
    core.hear_rex(RexNews::Linked(RexLink::Connected), 100);
    core.take_agent_reading(claude_in_a_pane(), 101);
    let recorded = fs::read_to_string(&file.0).unwrap_or_default();

    core.take_agent_reading(rex_gone(), 150);
    core.hear_rex(RexNews::Linked(RexLink::Reconnecting), 151);
    core.take_agent_reading(AgentSources::default(), 160);

    assert!(recorded.contains(&pane().block), "{recorded}");
    assert_eq!(fs::read_to_string(&file.0).unwrap_or_default(), recorded);
}
