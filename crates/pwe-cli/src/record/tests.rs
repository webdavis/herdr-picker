use std::fs;
use std::path::PathBuf;

use super::*;
use crate::socket_server::{Reply, SocketServer};

const PANE: &str = r#"{"id":"req_1","result":{"type":"pane_info","pane":{"pane_id":"w1:p3","workspace_id":"w1","tab_id":"w1:t2","cwd":"/work/repo","agent":null,"agent_status":"unknown"}}}"#;

struct TempState(PathBuf);

impl Drop for TempState {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn a_focused_pane_is_logged_with_its_tab_and_folder_as_herdr_reports_them() {
    let state = TempState(std::env::temp_dir().join(format!("hp-record-{}", std::process::id())));
    let places = state.0.join("places");
    let server = SocketServer::start(vec![Reply::Line(PANE.into())]);

    remember_pane(&server.path, "w1:p3", &places, 1_700_000_000);

    assert_eq!(server.seen()["params"]["pane_id"], "w1:p3");
    assert_eq!(
        fs::read_to_string(&places).expect("the places log"),
        "1700000000\tw1\tw1:t2\tw1:p3\t/work/repo\n"
    );
}

#[test]
fn a_pane_herdr_cannot_describe_is_not_logged() {
    let state =
        TempState(std::env::temp_dir().join(format!("hp-record-gone-{}", std::process::id())));
    let places = state.0.join("places");
    let server = SocketServer::start(vec![Reply::Line(
        r#"{"id":"req_1","error":{"code":"not_found","message":"no such pane"}}"#.into(),
    )]);

    remember_pane(&server.path, "w1:p3", &places, 1);

    assert!(!places.exists());
}
