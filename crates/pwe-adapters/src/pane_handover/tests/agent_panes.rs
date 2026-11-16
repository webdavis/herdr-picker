use std::time::Duration;

use pwe_protocol::Refusal;

use super::{DRAFT_CLAUDE, EMPTY_CLAUDE, hand, method_and_params, ok, pane, screen};
use crate::pane_handover::{HandoverError, SocketPanes, hand_over};
use crate::socket_server::SocketServer;

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
