use super::{hand, method_and_params, ok, pane};
use crate::socket_server::SocketServer;

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
