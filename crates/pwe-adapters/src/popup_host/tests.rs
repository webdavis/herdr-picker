use super::*;
use crate::socket_server::{Reply, SocketServer};
use serde_json::json;

const LAYOUT: &str = r#"{"id":"req_1","result":{"type":"pane_layout","layout":{"workspace_id":"w6","area":{"x":0,"y":0,"width":175,"height":65}}}}"#;
const OK: &str = r#"{"id":"req_1","result":{"type":"ok"}}"#;
const BUSY: &str = r#"{"id":"req_1","error":{"code":"ui_busy","message":"another modal is open"}}"#;
const SIZE: Size = Size {
    columns: 129,
    rows: 40,
};

fn host(server: &SocketServer, current_workspace: Option<&str>) -> SocketPopupHost {
    SocketPopupHost::at(
        &server.path,
        Duration::from_millis(150),
        "herdr-picker",
        protocol::PopupPane::workspace_picker(current_workspace),
    )
}

#[test]
fn the_window_size_is_the_area_of_the_focused_panes_layout() {
    let server = SocketServer::start(vec![Reply::Line(LAYOUT.into())]);

    assert_eq!(
        host(&server, None).window_size(),
        Ok(Size {
            columns: 175,
            rows: 65
        })
    );
    assert_eq!(server.seen()["method"], "pane.layout");
}

#[test]
fn opening_asks_herdr_for_this_plugins_pick_popup_with_the_current_workspace() {
    let server = SocketServer::start(vec![Reply::Line(OK.into())]);

    assert_eq!(host(&server, Some("w6")).open(SIZE), Ok(()));
    let request = server.seen();
    assert_eq!(request["method"], "plugin.pane.open");
    assert_eq!(
        request["params"],
        json!({
            "plugin_id": "herdr-picker",
            "entrypoint": "pick",
            "placement": "popup",
            "width": 129,
            "height": 40,
            "env": {"HERDR_ACTIVE_WORKSPACE_ID": "w6"}
        })
    );
}

#[test]
fn a_herdr_busy_with_another_modal_refuses_the_popup() {
    let server = SocketServer::start(vec![Reply::Line(BUSY.into())]);

    assert_eq!(
        host(&server, None).open(SIZE),
        Err(JumpError::Server {
            code: "ui_busy".to_string(),
            message: "another modal is open".to_string(),
        })
    );
}
