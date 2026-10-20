use std::time::Duration;

use herdr_picker_protocol::Refusal;

use super::*;
use crate::socket_server::{Reply, SocketServer};

const OK: &str = r#"{"id":"req_1","result":{"type":"ok"}}"#;
pub(super) const EMPTY_CLAUDE: &str = "  Done.\n───\n❯ \n───\n";
pub(super) const DRAFT_CLAUDE: &str = "  Done.\n───\n❯ fix the flaky\n───\n";

pub(super) fn pane(pane_id: &str, agent: Option<&str>, status: &str) -> Reply {
    let agent = agent.map_or("null".to_string(), |agent| format!("\"{agent}\""));
    Reply::Line(format!(
        r#"{{"id":"req_1","result":{{"type":"pane_info","pane":{{"pane_id":"{pane_id}","workspace_id":"w1","tab_id":"w1:t1","cwd":"/nowhere/repo","agent":{agent},"agent_status":"{status}"}}}}}}"#
    ))
}

pub(super) fn screen(text: &str) -> Reply {
    let text = serde_json::to_string(text).expect("JSON text");
    Reply::Line(format!(
        r#"{{"id":"req_1","result":{{"type":"pane_read","read":{{"text":{text}}}}}}}"#
    ))
}

pub(super) fn ok() -> Reply {
    Reply::Line(OK.into())
}

fn hand(server: &SocketServer, line: &str) -> Result<(), HandoverError> {
    let mut panes = SocketPanes::at(&server.path, Duration::from_millis(150));
    hand_over(
        &mut panes,
        "w1:p2",
        line,
        "nvim",
        Path::new("/nowhere/queue"),
    )
}

pub(super) fn method_and_params(server: &SocketServer) -> (String, serde_json::Value) {
    let request = server.seen();
    (
        request["method"].as_str().unwrap_or_default().to_string(),
        request["params"].clone(),
    )
}

#[test]
fn a_draft_in_claudes_input_line_stops_the_result_before_anything_is_typed() {
    let server = SocketServer::start(vec![
        pane("w1:p2", Some("claude"), "idle"),
        screen(DRAFT_CLAUDE),
    ]);

    assert_eq!(
        hand(&server, "run\tgit status"),
        Err(HandoverError::Refused(Refusal::Draft(
            "fix the flaky".to_string()
        )))
    );
    assert_eq!(method_and_params(&server).0, "pane.get");
    assert_eq!(method_and_params(&server).0, "pane.read");
}

#[test]
fn a_run_over_an_idle_claude_is_sent_behind_a_bang() {
    let server = SocketServer::start(vec![
        pane("w1:p2", Some("claude"), "idle"),
        screen(EMPTY_CLAUDE),
        ok(),
        ok(),
    ]);

    assert_eq!(hand(&server, "run\tgit status"), Ok(()));
    let _ = (server.seen(), server.seen());
    assert_eq!(
        method_and_params(&server),
        (
            "pane.send_text".to_string(),
            serde_json::json!({"pane_id": "w1:p2", "text": "! git status"})
        )
    );
    assert_eq!(method_and_params(&server).0, "pane.send_keys");
}

#[test]
fn a_folder_over_a_shell_with_an_empty_prompt_is_changed_into() {
    let server = SocketServer::start(vec![
        pane("w1:p2", None, "unknown"),
        screen("~/repo\n❯ \n"),
        ok(),
        ok(),
    ]);

    assert_eq!(hand(&server, "cd\tsrc"), Ok(()));
    let _ = (server.seen(), server.seen());
    assert_eq!(
        method_and_params(&server),
        (
            "pane.send_text".to_string(),
            serde_json::json!({"pane_id": "w1:p2", "text": "cd src"})
        )
    );
    assert_eq!(method_and_params(&server).0, "pane.send_keys");
}

#[test]
fn a_shell_draft_refuses_a_folder_or_run_but_takes_an_insert_after_it() {
    let drafted = || {
        SocketServer::start(vec![
            pane("w1:p2", None, "unknown"),
            screen("~/repo\n❯ git sta\n"),
            ok(),
        ])
    };
    let refused = Err(HandoverError::Refused(Refusal::Draft(
        "git sta".to_string(),
    )));

    assert_eq!(hand(&drafted(), "cd\tsrc"), refused);
    assert_eq!(hand(&drafted(), "run\tjust test"), refused);
    let server = SocketServer::start(vec![pane("w1:p2", None, "unknown"), ok()]);
    assert_eq!(hand(&server, "insert\tsrc/main.rs"), Ok(()));
    let _ = server.seen();
    assert_eq!(
        method_and_params(&server),
        (
            "pane.send_text".to_string(),
            serde_json::json!({"pane_id": "w1:p2", "text": "src/main.rs"})
        )
    );
}

#[test]
fn a_new_pane_to_the_left_is_split_right_swapped_and_runs_the_command() {
    let server = SocketServer::start(vec![
        pane("w1:p2", Some("claude"), "working"),
        pane("w1:p7", None, "unknown"),
        ok(),
        ok(),
        ok(),
    ]);

    assert_eq!(hand(&server, "left\trun\tnvim a.rs"), Ok(()));
    let _ = server.seen();
    assert_eq!(
        method_and_params(&server),
        (
            "pane.split".to_string(),
            serde_json::json!({"target_pane_id": "w1:p2", "direction": "right", "cwd": "/nowhere/repo", "focus": true})
        )
    );
    assert_eq!(
        method_and_params(&server),
        (
            "pane.swap".to_string(),
            serde_json::json!({"source_pane_id": "w1:p7", "target_pane_id": "w1:p2"})
        )
    );
    assert_eq!(
        method_and_params(&server),
        (
            "pane.send_text".to_string(),
            serde_json::json!({"pane_id": "w1:p7", "text": "nvim a.rs"})
        )
    );
}

#[test]
fn a_folder_for_a_new_tab_opens_the_tab_there_and_types_nothing() {
    let server = SocketServer::start(vec![
        pane("w1:p2", None, "unknown"),
        pane("w1:p9", None, "unknown"),
    ]);

    assert_eq!(hand(&server, "tab\tcd\t/srv"), Ok(()));
    let _ = server.seen();
    assert_eq!(
        method_and_params(&server),
        (
            "tab.create".to_string(),
            serde_json::json!({"workspace_id": "w1", "cwd": "/srv", "focus": true})
        )
    );
}

#[test]
fn an_instruction_herdr_picker_cannot_read_is_refused_before_herdr_is_asked() {
    let server = SocketServer::start(vec![]);

    assert_eq!(
        hand(&server, "copy\tx"),
        Err(HandoverError::Unreadable("copy\tx".to_string()))
    );
}

#[test]
fn a_herdr_call_goes_to_herdr_and_types_nothing() {
    let server = SocketServer::start(vec![pane("w1:p2", None, "unknown"), ok()]);

    assert_eq!(hand(&server, "herdr\t{\"focus_pane\":\"w1:p7\"}"), Ok(()));
    assert_eq!(method_and_params(&server).0, "pane.get");
    assert_eq!(
        method_and_params(&server),
        (
            "pane.focus".to_string(),
            serde_json::json!({"pane_id": "w1:p7"})
        )
    );
}

#[test]
fn a_prompt_the_agent_cannot_take_yet_is_queued_and_herdr_says_so() {
    let server = SocketServer::start(vec![
        pane("w1:p2", None, "unknown"),
        pane("w1:p5", Some("claude"), "blocked"),
        screen(EMPTY_CLAUDE),
        ok(),
    ]);
    let queue = std::env::temp_dir().join(format!("hp-handover-queue-{}", std::process::id()));
    let mut panes = SocketPanes::at(&server.path, Duration::from_millis(150));
    let line = "herdr\t{\"prompt\":{\"pane_id\":\"w1:p5\",\"text\":\"hi\"}}";

    assert_eq!(hand_over(&mut panes, "w1:p2", line, "nvim", &queue), Ok(()));
    let _ = (server.seen(), server.seen(), server.seen());
    assert_eq!(
        method_and_params(&server),
        (
            "notification.show".to_string(),
            serde_json::json!({"title": "herdr-picker",
                "body": "Queued: it waits on an answer. It is sent once the agent is free."})
        )
    );
    let _ = std::fs::remove_file(&queue);
}
