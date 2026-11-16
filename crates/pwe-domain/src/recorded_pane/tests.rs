use super::*;
use crate::{AgentState, StateFrom};

fn session(program: ProgramName, pane: Option<&str>) -> AgentSession {
    AgentSession {
        program,
        session_id: "00000000-0000-4000-8000-000000000042".to_string(),
        name: None,
        folder: Some("/work/project".to_string()),
        account: None,
        pid: Some(4101),
        pane: pane.map(|block| Pane {
            rex_session: "session:one".to_string(),
            tab: "window:one".to_string(),
            block: block.to_string(),
        }),
        state: AgentState::Idle,
        waiting_for: None,
        state_since: 100,
        state_from: StateFrom::Rex,
    }
}

#[test]
fn only_sessions_in_a_rex_pane_are_recorded() {
    let recorded = recorded_panes(&[
        session(ProgramName::ClaudeCode, Some("block:a")),
        session(ProgramName::Codex, None),
    ]);

    assert_eq!(
        recorded,
        vec![RecordedPane {
            program: ProgramName::ClaudeCode,
            session_id: "00000000-0000-4000-8000-000000000042".to_string(),
            folder: Some("/work/project".to_string()),
            pane: Pane {
                rex_session: "session:one".to_string(),
                tab: "window:one".to_string(),
                block: "block:a".to_string(),
            },
        }]
    );
}
