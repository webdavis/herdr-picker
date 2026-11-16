use std::time::Duration;

use super::*;
use crate::socket_server::{Reply, SocketServer};

mod agent_panes;
mod fake_host;
mod new_panes;
mod shell_panes;

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
