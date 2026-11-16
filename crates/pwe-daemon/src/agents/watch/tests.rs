use super::*;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};

use pwe_domain::{AgentState, Pane, ProgramName, WaitingKind};
use pwe_protocol::{ProgramStatus, RexBlock, agent_sessions_value};

const SESSION: &str = "session:00000000-0000-7000-8000-0000000000a1";
const BLOCK: &str = "block:00000000-0000-7000-8000-0000000000b1";

#[derive(Clone, Default)]
struct Heard(Arc<Mutex<Vec<AgentChange>>>);

impl AgentSubscriber for Heard {
    fn hear(&mut self, change: &AgentChange) {
        if let Ok(mut changes) = self.0.lock() {
            changes.push(change.clone());
        }
    }
}

impl Heard {
    fn changes(&self) -> Vec<AgentChange> {
        self.0
            .lock()
            .map(|changes| changes.clone())
            .unwrap_or_default()
    }
}

fn watch() -> (AgentWatch, Receiver<()>, Heard) {
    let (asks, asked) = mpsc::sync_channel(1);
    let mut watch = AgentWatch::new(asks);
    let heard = Heard::default();
    watch.subscribe(Box::new(heard.clone()));
    (watch, asked, heard)
}

fn working_claude_in_a_pane() -> AgentSources {
    AgentSources {
        reports: vec![ProgramReport {
            program: ProgramName::ClaudeCode,
            session_id: "00000000-0000-4000-8000-000000000042".to_string(),
            name: Some("tidy the notes".to_string()),
            folder: Some("/work/project".to_string()),
            account: None,
            pid: Some(4101),
            state: AgentState::Working,
            waiting_for: None,
            state_since: None,
        }],
        panes: vec![PaneStatus {
            pane: Pane {
                rex_session: SESSION.to_string(),
                tab: "window:00000000-0000-7000-8000-0000000000c1".to_string(),
                block: BLOCK.to_string(),
            },
            program: Some(ProgramName::ClaudeCode),
            pid: Some(4101),
            folder: Some("/work/project".to_string()),
            state: AgentState::Working,
            waiting_for: None,
        }],
        refusals: Vec::new(),
    }
}

fn status_event(block: &str, state: &str) -> RexEvent {
    RexEvent::ProgramStatusChanged {
        at: RexBlock {
            rex_session: SESSION.to_string(),
            block: block.to_string(),
        },
        status: ProgramStatus {
            state: state.to_string(),
            kind: Some("permission".to_string()),
            program: "claude-code".to_string(),
            message: Some("approve Bash: ls".to_string()),
            pid: Some(4101),
        },
    }
}

#[test]
fn a_blocked_status_event_reaches_subscribers_as_waiting() {
    let (mut watch, _asked, heard) = watch();
    watch.take_reading(working_claude_in_a_pane(), 100);

    watch.hear_event(&status_event(BLOCK, "blocked"), 130);

    let Some(AgentChange::Changed(session)) = heard.changes().last().cloned() else {
        panic!("no change heard: {:?}", heard.changes());
    };
    assert_eq!(session.state, AgentState::Waiting);
    assert_eq!(session.state_since, 130);
    assert_eq!(
        session.waiting_for.map(|waiting| waiting.kind),
        Some(WaitingKind::Permission)
    );
}

#[test]
fn a_status_event_for_a_pane_not_read_yet_asks_for_a_reading() {
    let (mut watch, asked, _heard) = watch();
    watch.take_reading(working_claude_in_a_pane(), 100);

    watch.hear_event(
        &status_event("block:00000000-0000-7000-8000-0000000000b9", "working"),
        130,
    );

    assert!(asked.try_recv().is_ok());
}

#[test]
fn a_closed_pane_asks_for_a_reading_and_a_tab_change_does_not() {
    let (mut watch, asked, _heard) = watch();
    let at = RexBlock {
        rex_session: SESSION.to_string(),
        block: BLOCK.to_string(),
    };

    watch.hear_event(
        &RexEvent::TabChanged {
            rex_session: SESSION.to_string(),
            tab: "window:00000000-0000-7000-8000-0000000000c2".to_string(),
        },
        100,
    );
    let after_the_tab = asked.try_recv().is_ok();
    watch.hear_event(&RexEvent::BlockClosed { at }, 101);

    assert!(!after_the_tab);
    assert!(asked.try_recv().is_ok());
}

#[test]
fn there_is_no_listing_before_the_first_reading() {
    let (mut watch, _asked, _heard) = watch();
    let before = watch.listing();

    watch.take_reading(working_claude_in_a_pane(), 100);

    assert_eq!(before, None);
    let mut book = AgentBook::default();
    let sources = working_claude_in_a_pane();
    book.update(&sources.reports, &sources.panes, 100);
    assert_eq!(
        watch.listing(),
        agent_sessions_value(&book.newest_state_first()).ok()
    );
}
