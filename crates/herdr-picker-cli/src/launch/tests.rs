use std::path::Path;

use serde_json::json;

use super::*;
use crate::socket_server::{Reply, SocketServer};

const LAYOUT: &str = r#"{"id":"req_1","result":{"type":"pane_layout","layout":{"area":{"x":0,"y":0,"width":175,"height":65}}}}"#;
const OK: &str = r#"{"id":"req_2","result":{"type":"ok"}}"#;
const CONTEXT: &str =
    r#"{"focused_pane_id":"w1:p3","focused_pane_cwd":"/work/repo","focused_pane_agent":"claude"}"#;

fn no_config() -> &'static Path {
    Path::new("/nonexistent/herdr-picker/config.toml")
}

#[test]
fn the_launcher_opens_over_the_focused_pane_in_its_folder_knowing_its_base() {
    let server = SocketServer::start(vec![Reply::Line(LAYOUT.into()), Reply::Line(OK.into())]);

    let opened = open(
        Some(server.path.to_string_lossy().into_owned()),
        Some("herdr-picker".to_string()),
        Some(CONTEXT.to_string()),
        "ctrl-g",
        no_config(),
    );

    assert!(opened.is_ok(), "{opened:?}");
    assert_eq!(server.seen()["method"], "pane.layout");
    let popup = server.seen();
    assert_eq!(popup["method"], "plugin.pane.open");
    assert_eq!(popup["params"]["entrypoint"], "launcher");
    assert_eq!(popup["params"]["cwd"], "/work/repo");
    assert_eq!(
        popup["params"]["env"],
        json!({"HERDR_PICKER_BASE": "ctrl-g", "HERDR_PICKER_TARGET": "w1:p3"})
    );
}

#[test]
fn a_base_that_is_not_one_key_or_a_context_without_a_pane_is_refused_before_herdr_is_asked() {
    let socket = Some("/nonexistent/herdr.sock".to_string());
    let plugin = Some("herdr-picker".to_string());

    assert!(matches!(
        open(
            socket.clone(),
            plugin.clone(),
            Some(CONTEXT.to_string()),
            "ctrl-g b",
            no_config()
        ),
        Err(CommandError::Usage)
    ));
    assert!(matches!(
        open(socket, plugin, Some("{}".to_string()), "ctrl-g", no_config()),
        Err(CommandError::Failed(reason)) if reason.contains("no focused pane")
    ));
}
