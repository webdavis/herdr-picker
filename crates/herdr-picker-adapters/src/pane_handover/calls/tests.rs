use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use herdr_picker_domain::Agent;
use serde_json::json;

use super::*;
use crate::pane_handover::tests::{
    DRAFT_CLAUDE, EMPTY_CLAUDE, method_and_params, ok, pane, screen,
};
use crate::socket_server::{Reply, SocketServer};

struct TempQueue(PathBuf);

impl TempQueue {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("hp-queue-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        Self(dir)
    }

    fn path(&self) -> PathBuf {
        self.0.join("send-queue.json")
    }
}

impl Drop for TempQueue {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn panes(server: &SocketServer) -> SocketPanes {
    SocketPanes::at(&server.path, Duration::from_millis(150))
}

fn queued(pane_id: &str, text: &str) -> Queued {
    Queued {
        pane_id: pane_id.to_string(),
        text: text.to_string(),
    }
}

#[test]
fn a_working_agent_with_an_empty_input_line_is_prompted_at_once() {
    let queue = TempQueue::new("now");
    let server = SocketServer::start(vec![
        pane("w1:p2", Some("claude"), "working"),
        screen(EMPTY_CLAUDE),
        ok(),
    ]);

    let sent = send_prompt(&mut panes(&server), &queue.path(), "w1:p2", "look at this");

    assert_eq!(sent, Ok(Sent::Now));
    let _ = (server.seen(), server.seen());
    assert_eq!(
        method_and_params(&server),
        (
            "agent.prompt".to_string(),
            json!({"target": "w1:p2", "text": "look at this"})
        )
    );
    assert_eq!(read_queue(&queue.path()), Vec::new());
}

#[test]
fn the_owners_draft_queues_the_prompt_and_nothing_is_typed() {
    let queue = TempQueue::new("draft");
    let server = SocketServer::start(vec![
        pane("w1:p2", Some("claude"), "idle"),
        screen(DRAFT_CLAUDE),
    ]);

    let sent = send_prompt(&mut panes(&server), &queue.path(), "w1:p2", "later");

    assert_eq!(
        sent,
        Ok(Sent::Queued(Refusal::Draft("fix the flaky".to_string())))
    );
    assert_eq!(read_queue(&queue.path()), vec![queued("w1:p2", "later")]);
}

#[test]
fn a_blocked_agent_queues_the_prompt_without_reading_further() {
    let queue = TempQueue::new("blocked");
    let server = SocketServer::start(vec![
        pane("w1:p2", Some("codex"), "blocked"),
        screen(EMPTY_CLAUDE),
    ]);

    let sent = send_prompt(&mut panes(&server), &queue.path(), "w1:p2", "later");

    assert_eq!(sent, Ok(Sent::Queued(Refusal::Blocked)));
}

#[test]
fn an_agent_that_settles_with_an_empty_line_takes_its_oldest_queued_prompt() {
    let queue = TempQueue::new("drain");
    let waiting = [
        queued("w1:p2", "first"),
        queued("w9:p1", "other"),
        queued("w1:p2", "second"),
    ];
    write_queue(&queue.path(), &waiting).expect("saved");
    let server = SocketServer::start(vec![
        pane("w1:p2", Some("claude"), "idle"),
        screen(EMPTY_CLAUDE),
        ok(),
    ]);

    let drained = drain_queue(
        &mut panes(&server),
        &queue.path(),
        "w1:p2",
        AgentStatus::Idle,
    );

    assert_eq!(drained, Ok(true));
    let _ = (server.seen(), server.seen());
    assert_eq!(
        method_and_params(&server),
        (
            "agent.prompt".to_string(),
            json!({"target": "w1:p2", "text": "first"})
        )
    );
    assert_eq!(
        read_queue(&queue.path()),
        vec![queued("w9:p1", "other"), queued("w1:p2", "second")]
    );
}

#[test]
fn two_hooks_draining_at_once_send_each_queued_prompt_once() {
    let queue = TempQueue::new("race");
    write_queue(
        &queue.path(),
        &[queued("w1:p2", "first"), queued("w1:p2", "second")],
    )
    .expect("saved");
    let start = std::sync::Barrier::new(2);

    let sent: Vec<String> = std::thread::scope(|scope| {
        let hooks: Vec<_> = (0..2)
            .map(|_| {
                scope.spawn(|| {
                    let server = SocketServer::start(vec![
                        pane("w1:p2", Some("claude"), "idle"),
                        screen(EMPTY_CLAUDE),
                        ok(),
                    ]);
                    start.wait();
                    let drained = drain_queue(
                        &mut panes(&server),
                        &queue.path(),
                        "w1:p2",
                        AgentStatus::Idle,
                    );
                    assert_eq!(drained, Ok(true));
                    let _ = (server.seen(), server.seen());
                    method_and_params(&server).1["text"].to_string()
                })
            })
            .collect();
        hooks
            .into_iter()
            .map(|hook| hook.join().expect("a hook"))
            .collect()
    });

    let mut sent = sent;
    sent.sort();
    assert_eq!(sent, vec!["\"first\"", "\"second\""]);
    assert_eq!(read_queue(&queue.path()), Vec::new());
}

#[test]
fn a_draft_or_a_turn_still_running_leaves_the_queue_alone() {
    let queue = TempQueue::new("hold");
    write_queue(&queue.path(), &[queued("w1:p2", "first")]).expect("saved");
    let server = SocketServer::start(vec![
        pane("w1:p2", Some("claude"), "done"),
        screen(DRAFT_CLAUDE),
    ]);

    let working = drain_queue(
        &mut panes(&server),
        &queue.path(),
        "w1:p2",
        AgentStatus::Working,
    );
    let drafted = drain_queue(
        &mut panes(&server),
        &queue.path(),
        "w1:p2",
        AgentStatus::Done,
    );

    assert_eq!((working, drafted), (Ok(false), Ok(false)));
    assert_eq!(read_queue(&queue.path()), vec![queued("w1:p2", "first")]);
}

#[test]
fn a_snapshot_asks_herdr_for_its_agents_workspaces_and_tabs() {
    let server = SocketServer::start(vec![
        Reply::Line(
            json!({"id": "req_1", "result": {"type": "agent_list", "agents": [{
                "agent": "claude", "agent_status": "idle", "cwd": "/repo", "pane_id": "w1:p1",
                "tab_id": "w1:t1", "workspace_id": "w1", "terminal_title_stripped": "fix"}]}})
            .to_string(),
        ),
        Reply::Line(
            json!({"id": "req_2", "result": {"type": "workspace_list", "workspaces": [
                {"workspace_id": "w1", "label": "dotfiles"}]}})
            .to_string(),
        ),
        Reply::Line(
            json!({"id": "req_3", "result": {"type": "tab_list", "tabs": [
                {"tab_id": "w1:t1", "workspace_id": "w1", "label": "keys"}]}})
            .to_string(),
        ),
    ]);

    let snapshot = panes(&server).snapshot().expect("a snapshot");

    assert_eq!(
        (
            snapshot
                .agents
                .iter()
                .map(|agent: &Agent| agent.title.as_str())
                .collect::<Vec<_>>(),
            snapshot.workspaces[0].label.as_str(),
            snapshot.tabs[0].label.as_str(),
        ),
        (vec!["fix"], "dotfiles", "keys")
    );
}
