use serde_json::json;

use super::*;

fn sent(line: &str) -> Value {
    serde_json::from_str(line.trim()).expect("a JSON request")
}

#[test]
fn a_pane_is_read_from_a_pane_info_or_a_new_tabs_root_pane() {
    let info = json!({"type": "pane_info", "pane": {
        "pane_id": "w1:p2", "workspace_id": "w1", "tab_id": "w1:t1",
        "cwd": "/work", "agent": "claude", "agent_status": "idle"}});
    let tab = json!({"type": "tab_created", "tab": {}, "root_pane": {
        "pane_id": "w1:p9", "workspace_id": "w1", "tab_id": "w1:t3",
        "cwd": null, "agent": null, "agent_status": "unknown"}});

    assert_eq!(
        pane_of(&info),
        Ok(PaneInfo {
            pane_id: "w1:p2".to_string(),
            workspace_id: "w1".to_string(),
            tab_id: "w1:t1".to_string(),
            cwd: "/work".to_string(),
            agent: Some("claude".to_string()),
            status: "idle".to_string(),
        })
    );
    assert_eq!(
        pane_of(&tab).map(|pane| pane.pane_id),
        Ok("w1:p9".to_string())
    );
    assert!(pane_of(&json!({"type": "ok"})).is_err());
}

#[test]
fn typing_sends_the_text_and_submitting_presses_enter_after_it() {
    assert_eq!(
        sent(&type_request("req_1", "w1:p2", "! ls")),
        json!({"id": "req_1", "method": "pane.send_text",
               "params": {"pane_id": "w1:p2", "text": "! ls"}})
    );
    assert_eq!(
        sent(&enter_request("req_2", "w1:p2")),
        json!({"id": "req_2", "method": "pane.send_keys",
               "params": {"pane_id": "w1:p2", "keys": ["enter"]}})
    );
}

#[test]
fn the_screen_is_the_visible_text() {
    assert_eq!(
        sent(&screen_request("req_1", "w1:p2"))["params"],
        json!({"pane_id": "w1:p2", "source": "visible"})
    );
    assert_eq!(
        screen_of(&json!({"type": "pane_read", "read": {"text": "❯ hi\n"}})),
        Ok("❯ hi\n".to_string())
    );
}

#[test]
fn a_new_pane_splits_right_or_down_and_a_new_tab_opens_in_the_same_workspace() {
    let beside = PaneInfo {
        pane_id: "w1:p2".to_string(),
        workspace_id: "w1".to_string(),
        tab_id: "w1:t1".to_string(),
        cwd: "/work".to_string(),
        agent: None,
        status: "unknown".to_string(),
    };
    let opened = |pane| sent(&open_request("req_1", &beside, pane, "/srv"));

    let split = |direction| {
        json!({"id": "req_1", "method": "pane.split", "params": {
            "target_pane_id": "w1:p2", "direction": direction, "cwd": "/srv", "focus": true}})
    };
    assert_eq!(opened(NewPane::Left), split("right"));
    assert_eq!(opened(NewPane::Right), split("right"));
    assert_eq!(opened(NewPane::Above), split("down"));
    assert_eq!(opened(NewPane::Below), split("down"));
    assert_eq!(
        opened(NewPane::Tab),
        json!({"id": "req_1", "method": "tab.create", "params": {
            "workspace_id": "w1", "cwd": "/srv", "focus": true}})
    );
    assert_eq!(
        sent(&swap_request("req_2", "w1:p7", "w1:p2"))["params"],
        json!({"source_pane_id": "w1:p7", "target_pane_id": "w1:p2"})
    );
}

#[test]
fn a_note_shows_as_a_herdr_notification() {
    assert_eq!(
        sent(&notify_request("req_1", "Claude: nothing was sent")),
        json!({"id": "req_1", "method": "notification.show", "params": {
            "title": "herdr-picker", "body": "Claude: nothing was sent"}})
    );
}
