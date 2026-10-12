use super::*;
use crate::socket_server::{Reply, SocketServer};
use serde_json::json;

const LAYOUT: &str = r#"{"id":"req_1","result":{"type":"pane_layout","layout":{"area":{"x":0,"y":0,"width":58,"height":29}}}}"#;
const OK: &str = r#"{"id":"req_2","result":{"type":"ok"}}"#;

fn workspace(label: &str, pick_key: char) -> JumpTarget {
    JumpTarget {
        action_id: format!("jump_{label}"),
        label: label.to_string(),
        directory: format!("/opt/{label}"),
        pick_key,
    }
}

#[test]
fn the_picker_opens_wide_enough_for_the_longest_workspace_and_knows_where_it_came_from() {
    let server = SocketServer::start(vec![Reply::Line(LAYOUT.into()), Reply::Line(OK.into())]);

    let opened = run(
        Some(server.path.to_string_lossy().into_owned()),
        Some("herdr-workspace-jump".to_string()),
        Some("w6".to_string()),
        &[
            workspace("Ivy", 'i'),
            workspace("essential-feed-case-study", 'e'),
        ],
    );

    assert!(opened.is_ok(), "{opened:?}");
    assert_eq!(server.seen()["method"], "pane.layout");
    let open = server.seen();
    assert_eq!(open["params"]["plugin_id"], "herdr-workspace-jump");
    assert_eq!(open["params"]["width"], json!(43));
    assert_eq!(open["params"]["height"], json!(17));
    assert_eq!(open["params"]["env"]["HERDR_ACTIVE_WORKSPACE_ID"], "w6");
}

#[test]
fn the_picker_needs_herdrs_socket_and_this_plugins_id_and_says_which_is_missing() {
    let without_socket = run(None, Some("herdr-workspace-jump".to_string()), None, &[]);
    let without_plugin = run(Some("/nonexistent/hwj.sock".to_string()), None, None, &[]);

    assert!(
        matches!(&without_socket, Err(CommandError::Failed(reason)) if reason.contains("HERDR_SOCKET_PATH")),
        "{without_socket:?}"
    );
    assert!(
        matches!(&without_plugin, Err(CommandError::Failed(reason)) if reason.contains("HERDR_PLUGIN_ID")),
        "{without_plugin:?}"
    );
}
