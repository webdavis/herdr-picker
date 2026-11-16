use super::*;
use std::collections::HashMap;

use pwe_domain::{Pane, ProgramName};

const CLAUDE_ID: &str = "00000000-0000-4000-8000-000000000042";

#[derive(Default)]
struct FakePanes {
    now: HashMap<String, Vec<PaneNow>>,
    typed: Vec<(String, String)>,
}

impl FakePanes {
    fn showing(mut self, block: &str, states: &[PaneNow]) -> Self {
        self.now.insert(block.to_string(), states.to_vec());
        self
    }
}

impl RestoredPanes for FakePanes {
    fn pane_now(&mut self, block: &str) -> PaneNow {
        let states = self.now.entry(block.to_string()).or_default();
        match states.len() {
            0 => PaneNow::NotReady,
            1 => states[0].clone(),
            _ => states.remove(0),
        }
    }

    fn type_line(&mut self, block: &str, line: &str) {
        self.typed.push((block.to_string(), line.to_string()));
    }
}

fn shell_in(folder: &str) -> PaneNow {
    PaneNow::ShellWaits {
        folder: Some(folder.to_string()),
    }
}

fn recorded(program: ProgramName, session_id: &str, block: &str) -> RecordedPane {
    RecordedPane {
        program,
        session_id: session_id.to_string(),
        folder: Some("/work/project".to_string()),
        pane: Pane {
            rex_session: "session:00000000-0000-7000-8000-0000000000a1".to_string(),
            tab: "window:00000000-0000-7000-8000-0000000000c1".to_string(),
            block: block.to_string(),
        },
    }
}

fn reopened(records: Vec<RecordedPane>, panes: &mut FakePanes) {
    reopen_recorded(records, panes, || true, 5);
}

#[test]
fn a_recorded_claude_pane_is_typed_its_resume_command() {
    let mut panes = FakePanes::default().showing("block:a", &[shell_in("/work/project")]);

    reopened(
        vec![recorded(ProgramName::ClaudeCode, CLAUDE_ID, "block:a")],
        &mut panes,
    );

    assert_eq!(
        panes.typed,
        vec![(
            "block:a".to_string(),
            format!("claude --resume {CLAUDE_ID}")
        )]
    );
}

#[test]
fn a_pane_running_another_program_is_never_typed_into() {
    let mut panes = FakePanes::default().showing("block:a", &[PaneNow::RunsAProgram]);

    reopened(
        vec![recorded(ProgramName::ClaudeCode, CLAUDE_ID, "block:a")],
        &mut panes,
    );

    assert!(panes.typed.is_empty(), "{:?}", panes.typed);
}

#[test]
fn a_pane_not_ready_yet_is_typed_into_once_its_shell_waits() {
    let mut panes = FakePanes::default().showing(
        "block:a",
        &[
            PaneNow::NotReady,
            PaneNow::NotReady,
            shell_in("/work/project"),
        ],
    );

    reopened(
        vec![recorded(ProgramName::ClaudeCode, CLAUDE_ID, "block:a")],
        &mut panes,
    );

    assert_eq!(panes.typed.len(), 1, "{:?}", panes.typed);
}

#[test]
fn an_omp_pane_is_left_alone_because_omp_shows_no_resume_command() {
    let mut panes = FakePanes::default().showing("block:a", &[shell_in("/work/project")]);

    reopened(
        vec![recorded(ProgramName::Omp, CLAUDE_ID, "block:a")],
        &mut panes,
    );

    assert!(panes.typed.is_empty(), "{:?}", panes.typed);
}

#[test]
fn a_pane_in_another_folder_is_not_reopened() {
    let mut panes = FakePanes::default().showing("block:a", &[shell_in("/elsewhere")]);

    reopened(
        vec![recorded(ProgramName::ClaudeCode, CLAUDE_ID, "block:a")],
        &mut panes,
    );

    assert!(panes.typed.is_empty(), "{:?}", panes.typed);
}

#[test]
fn a_pane_whose_folder_cannot_be_read_is_not_reopened() {
    let mut panes =
        FakePanes::default().showing("block:a", &[PaneNow::ShellWaits { folder: None }]);

    reopened(
        vec![recorded(ProgramName::ClaudeCode, CLAUDE_ID, "block:a")],
        &mut panes,
    );

    assert!(panes.typed.is_empty(), "{:?}", panes.typed);
}

#[test]
fn a_record_without_a_folder_is_not_reopened() {
    let mut panes = FakePanes::default().showing("block:a", &[shell_in("/work/project")]);
    let mut claude = recorded(ProgramName::ClaudeCode, CLAUDE_ID, "block:a");
    claude.folder = None;

    reopened(vec![claude], &mut panes);

    assert!(panes.typed.is_empty(), "{:?}", panes.typed);
}

#[test]
fn the_typed_line_never_holds_the_folder() {
    let folder = "/work/odd'folder";
    let mut panes = FakePanes::default().showing("block:a", &[shell_in(folder)]);
    let mut claude = recorded(ProgramName::ClaudeCode, CLAUDE_ID, "block:a");
    claude.folder = Some(folder.to_string());

    reopened(vec![claude], &mut panes);

    assert_eq!(
        panes.typed,
        vec![(
            "block:a".to_string(),
            format!("claude --resume {CLAUDE_ID}")
        )]
    );
}

#[test]
fn one_conversation_recorded_in_two_panes_reopens_in_one() {
    let mut panes = FakePanes::default()
        .showing("block:a", &[shell_in("/work/project")])
        .showing("block:b", &[shell_in("/work/project")]);

    reopened(
        vec![
            recorded(ProgramName::ClaudeCode, CLAUDE_ID, "block:a"),
            recorded(ProgramName::ClaudeCode, CLAUDE_ID, "block:b"),
        ],
        &mut panes,
    );

    assert_eq!(panes.typed.len(), 1, "{:?}", panes.typed);
}
