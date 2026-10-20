use herdr_picker_domain::AgentStatus;
use serde_json::json;

use super::*;

fn sent(line: &str) -> Value {
    serde_json::from_str(line.trim()).expect("a JSON request")
}

// Trimmed from what `herdr agent list` answered on 2026-10-19.
const LISTED: &str = r#"{"type":"agent_list","agents":[
  {"agent":"claude","agent_status":"working","cwd":"/repo","display_agent":"main","focused":true,
   "pane_id":"w1:pH","tab_id":"w1:tA","terminal_title":"◑ chezmoidata-refactor",
   "terminal_title_stripped":"chezmoidata-refactor","workspace_id":"w1"},
  {"agent":"codex","agent_status":"done","cwd":"/kb","pane_id":"w5:p3D","tab_id":"w5:tT",
   "workspace_id":"w5"}]}"#;

#[test]
fn listed_agents_carry_their_title_place_and_status() {
    let result: Value = serde_json::from_str(LISTED).expect("fixture JSON");

    assert_eq!(
        agents_of(&result),
        Ok(vec![
            Agent {
                pane_id: "w1:pH".to_string(),
                workspace_id: "w1".to_string(),
                tab_id: "w1:tA".to_string(),
                kind: "claude".to_string(),
                title: "chezmoidata-refactor".to_string(),
                cwd: "/repo".to_string(),
                status: AgentStatus::Working,
                since: None,
            },
            Agent {
                pane_id: "w5:p3D".to_string(),
                workspace_id: "w5".to_string(),
                tab_id: "w5:tT".to_string(),
                kind: "codex".to_string(),
                title: "codex".to_string(),
                cwd: "/kb".to_string(),
                status: AgentStatus::Done,
                since: None,
            },
        ])
    );
    assert!(agents_of(&json!({"type": "ok"})).is_err());
}

#[test]
fn listed_tabs_carry_their_label_and_workspace() {
    let result = json!({"type": "tab_list", "tabs": [
        {"tab_id": "w1:tA", "workspace_id": "w1", "label": "Improve Key Layout", "number": 10}]});

    assert_eq!(
        tabs_of(&result),
        Ok(vec![WireTab {
            tab_id: "w1:tA".to_string(),
            workspace_id: "w1".to_string(),
            label: "Improve Key Layout".to_string(),
        }])
    );
}

#[test]
fn each_call_names_herdrs_method_and_target() {
    let requests = [
        call_request("r", &HerdrCall::FocusPane("w1:p2".to_string())),
        call_request("r", &HerdrCall::FocusTab("w1:t2".to_string())),
        call_request("r", &HerdrCall::FocusWorkspace("w2".to_string())),
        call_request("r", &HerdrCall::CloseTab("w1:t2".to_string())),
        call_request(
            "r",
            &HerdrCall::Prompt {
                pane_id: "w1:p2".to_string(),
                text: "line one\nline two".to_string(),
            },
        ),
    ];

    assert_eq!(
        requests.map(|request| sent(&request)).to_vec(),
        vec![
            json!({"id": "r", "method": "pane.focus", "params": {"pane_id": "w1:p2"}}),
            json!({"id": "r", "method": "tab.focus", "params": {"tab_id": "w1:t2"}}),
            json!({"id": "r", "method": "workspace.focus", "params": {"workspace_id": "w2"}}),
            json!({"id": "r", "method": "tab.close", "params": {"tab_id": "w1:t2"}}),
            json!({"id": "r", "method": "agent.prompt",
                   "params": {"target": "w1:p2", "text": "line one\nline two"}}),
        ]
    );
}

#[test]
fn the_lists_ask_for_every_agent_and_every_tab() {
    assert_eq!(
        (sent(&agent_list_request("a")), sent(&tab_list_request("b"))),
        (
            json!({"id": "a", "method": "agent.list", "params": {}}),
            json!({"id": "b", "method": "tab.list", "params": {}})
        )
    );
}

#[test]
fn a_status_change_names_its_pane_and_new_status() {
    let event = r#"{"event":"pane_agent_status_changed","data":{"type":"pane_agent_status_changed",
        "pane_id":"w1:p1","workspace_id":"w1","agent_status":"idle","agent":"claude"}}"#;

    assert_eq!(
        parse_status_change(event),
        Some(("w1:p1".to_string(), AgentStatus::Idle))
    );
    assert_eq!(parse_status_change(r#"{"data":{}}"#), None);
}
