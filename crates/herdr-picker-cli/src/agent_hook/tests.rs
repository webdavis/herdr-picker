use std::fs;

use herdr_picker_domain::AgentStatus;
use serde_json::json;

use super::*;
use crate::socket_server::{Reply, SocketServer};

struct TempState(PathBuf);

impl Drop for TempState {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn listed(status: &str) -> Vec<Reply> {
    vec![
        Reply::Line(
            json!({"id": "req_1", "result": {"type": "agent_list", "agents": [{
                "agent": "claude", "agent_status": status, "cwd": "/repo", "pane_id": "w1:p1",
                "tab_id": "w1:t1", "workspace_id": "w1", "terminal_title_stripped": "fix"}]}})
            .to_string(),
        ),
        Reply::Line(json!({"id": "req_2", "result": {"workspaces": []}}).to_string()),
        Reply::Line(json!({"id": "req_3", "result": {"tabs": []}}).to_string()),
    ]
}

fn statuses(snapshot: &AgentSnapshot) -> Vec<(AgentStatus, Option<u64>)> {
    snapshot
        .agents
        .iter()
        .map(|agent| (agent.status, agent.since))
        .collect()
}

#[test]
fn the_agent_whose_status_changed_starts_its_clock_and_the_list_is_saved() {
    let state = TempState(std::env::temp_dir().join(format!("hp-hook-{}", std::process::id())));
    let saved = state.0.join("agents.json");
    let server = SocketServer::start(listed("idle"));

    let refreshed = refresh(
        &mut SocketPanes::at(&server.path, HOOK_CAP),
        &saved,
        "w1:p1",
        500,
    );

    assert_eq!(
        refreshed.as_ref().map(statuses),
        Ok(vec![(AgentStatus::Idle, Some(500))])
    );
    let reread = read_snapshot(&saved, Duration::from_secs(60), SystemTime::now());
    assert_eq!(
        reread.as_ref().map(statuses),
        Some(vec![(AgentStatus::Idle, Some(500))])
    );
}

#[test]
fn another_agents_change_keeps_this_ones_clock() {
    let state =
        TempState(std::env::temp_dir().join(format!("hp-hook-keep-{}", std::process::id())));
    let saved = state.0.join("agents.json");
    let first = SocketServer::start(listed("working"));
    let _ = refresh(
        &mut SocketPanes::at(&first.path, HOOK_CAP),
        &saved,
        "w1:p1",
        100,
    );
    let second = SocketServer::start(listed("working"));

    let refreshed = refresh(
        &mut SocketPanes::at(&second.path, HOOK_CAP),
        &saved,
        "w9:p9",
        900,
    );

    assert_eq!(
        refreshed.as_ref().map(statuses),
        Ok(vec![(AgentStatus::Working, Some(100))])
    );
}
