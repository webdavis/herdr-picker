use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use super::*;

struct TempState(PathBuf);

impl TempState {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("hp-agents-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        Self(dir)
    }
}

impl Drop for TempState {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn snapshot() -> AgentSnapshot {
    AgentSnapshot {
        agents: vec![Agent {
            pane_id: "w1:p1".to_string(),
            workspace_id: "w1".to_string(),
            tab_id: "w1:t1".to_string(),
            kind: "claude".to_string(),
            title: "refactor".to_string(),
            cwd: "/repo".to_string(),
            status: AgentStatus::Blocked,
            since: Some(1_700_000_000),
        }],
        workspaces: vec![Workspace {
            workspace_id: "w1".to_string(),
            label: "dotfiles".to_string(),
        }],
        tabs: vec![WireTab {
            tab_id: "w1:t1".to_string(),
            workspace_id: "w1".to_string(),
            label: "Improve keys".to_string(),
        }],
    }
}

#[test]
fn a_fresh_snapshot_reads_back_as_it_was_written() {
    let state = TempState::new("fresh");
    let path = state.0.join("agents.json");
    write_snapshot(&path, &snapshot()).expect("saved");

    assert_eq!(
        read_snapshot(&path, Duration::from_secs(60), SystemTime::now()),
        Some(snapshot())
    );
}

#[test]
fn a_stale_or_missing_snapshot_reads_as_none() {
    let state = TempState::new("stale");
    let path = state.0.join("agents.json");
    let later = SystemTime::now() + Duration::from_secs(120);

    assert_eq!(read_snapshot(&path, Duration::from_secs(60), later), None);
    write_snapshot(&path, &snapshot()).expect("saved");
    assert_eq!(read_snapshot(&path, Duration::from_secs(60), later), None);
}

#[test]
fn the_queue_keeps_its_prompts_in_order_and_only_its_owner_reads_it() {
    let state = TempState::new("queue");
    let path = state.0.join("send-queue.json");
    let queue = vec![
        Queued {
            pane_id: "w1:p1".to_string(),
            text: "first\nline".to_string(),
        },
        Queued {
            pane_id: "w2:p1".to_string(),
            text: "second".to_string(),
        },
    ];

    assert_eq!(read_queue(&path), Vec::new());
    write_queue(&path, &queue).expect("saved");
    let mode = std::os::unix::fs::PermissionsExt::mode(
        &fs::metadata(&path).expect("a file").permissions(),
    );

    assert_eq!((read_queue(&path), mode & 0o777), (queue, 0o600));
}
