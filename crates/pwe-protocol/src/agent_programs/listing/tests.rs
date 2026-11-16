use pwe_domain::{AgentSession, AgentState, Pane, ProgramName, StateFrom, WaitingFor, WaitingKind};
use serde_json::{Value, json};

use super::*;

fn waiting_session() -> AgentSession {
    AgentSession {
        program: ProgramName::ClaudeCode,
        session_id: "22222222".to_string(),
        name: Some("session-e".to_string()),
        folder: Some("/work/project".to_string()),
        account: Some("/home/someone/.claude-work".to_string()),
        pid: Some(101),
        pane: Some(Pane {
            rex_session: "session:one".to_string(),
            tab: "tab:one".to_string(),
            block: "block:one".to_string(),
        }),
        state: AgentState::Waiting,
        waiting_for: Some(WaitingFor {
            kind: WaitingKind::Permission,
            text: "permission prompt".to_string(),
        }),
        state_since: 1790000060,
        state_from: StateFrom::Rex,
    }
}

#[test]
fn a_session_prints_in_kebab_case_with_its_pane() {
    let printed = agent_sessions_json(&[waiting_session()]).expect("printed");

    let value: Value = serde_json::from_str(&printed).expect("JSON");
    assert_eq!(
        value,
        json!({"agent-sessions": [{
            "program": "claude-code",
            "session-id": "22222222",
            "name": "session-e",
            "folder": "/work/project",
            "account": "/home/someone/.claude-work",
            "pid": 101,
            "pane": {"rex-session": "session:one", "tab": "tab:one", "block": "block:one"},
            "state": "waiting",
            "waiting-for": {"kind": "permission", "text": "permission prompt"},
            "state-since": 1790000060,
            "state-from": "rex"
        }]})
    );
}

#[test]
fn unknown_fields_are_left_out() {
    let session = AgentSession {
        name: None,
        folder: None,
        account: None,
        pid: None,
        pane: None,
        waiting_for: None,
        ..waiting_session()
    };

    let printed = agent_sessions_json(&[session]).expect("printed");

    let value: Value = serde_json::from_str(&printed).expect("JSON");
    let keys: Vec<&String> = value["agent-sessions"][0]
        .as_object()
        .expect("an object")
        .keys()
        .collect();
    assert_eq!(
        keys,
        [
            "program",
            "session-id",
            "state",
            "state-since",
            "state-from"
        ]
    );
}

#[test]
fn no_sessions_print_an_empty_list() {
    assert_eq!(
        agent_sessions_json(&[]).expect("printed"),
        r#"{"agent-sessions":[]}"#
    );
}
