use super::*;
use crate::DEADLINE;
use crate::socket_server::{Reply, SocketServer};
use herdr_workspace_jump_application::jump;
use herdr_workspace_jump_domain::Jump;
use serde_json::Value;

enum AfterReplies {
    HangUp,
    StaySilent,
}

fn start_fake_herdr(replies: Vec<String>, after: AfterReplies) -> SocketServer {
    let mut replies: Vec<Reply> = replies.into_iter().map(Reply::Line).collect();
    if replies.is_empty() {
        replies.push(Reply::HangUp);
    }
    if matches!(after, AfterReplies::StaySilent) {
        replies.push(Reply::Silent);
    }
    SocketServer::start(replies)
}

fn connected(server: &SocketServer) -> SocketWorkspaceDirectory {
    SocketWorkspaceDirectory::at(&server.path, Duration::from_millis(150))
}

fn seen(server: &SocketServer) -> Value {
    server.seen()
}

const TWO_WORKSPACES: &str = r#"{"id":"req_1","result":{"workspaces":[{"workspace_id":"wA","label":"netpulse"},{"workspace_id":"w11","label":"dotfiles"}]}}"#;
const OK: &str = r#"{"id":"req_2","result":{"type":"ok"}}"#;

#[test]
fn a_jump_that_focuses_sends_list_then_focus_over_separate_connections() {
    let server = start_fake_herdr(vec![TWO_WORKSPACES.into(), OK.into()], AfterReplies::HangUp);
    let mut directory = connected(&server);
    let outcome = jump(&mut directory, "dotfiles", "/home/me/dotfiles");

    assert_eq!(outcome, Ok(Jump::Focused("w11".into())));
    assert_eq!(seen(&server)["method"], "workspace.list");
    let focus = seen(&server);
    assert_eq!(focus["method"], "workspace.focus");
    assert_eq!(focus["params"]["workspace_id"], "w11");
}

#[test]
fn a_jump_that_creates_sends_list_then_create_with_cwd_label_and_focus() {
    let server = start_fake_herdr(vec![TWO_WORKSPACES.into(), OK.into()], AfterReplies::HangUp);
    let mut directory = connected(&server);
    let outcome = jump(&mut directory, "homelab", "/home/me/homelab");

    assert_eq!(outcome, Ok(Jump::Created));
    assert_eq!(seen(&server)["method"], "workspace.list");
    let create = seen(&server);
    assert_eq!(create["method"], "workspace.create");
    assert_eq!(create["params"]["label"], "homelab");
    assert_eq!(create["params"]["cwd"], "/home/me/homelab");
    assert_eq!(create["params"]["focus"], true);
}

#[test]
fn each_request_carries_a_fresh_id() {
    let server = start_fake_herdr(vec![TWO_WORKSPACES.into(), OK.into()], AfterReplies::HangUp);
    let mut directory = connected(&server);
    let _ = jump(&mut directory, "dotfiles", "/tmp");
    assert_eq!(seen(&server)["id"], "req_1");
    assert_eq!(seen(&server)["id"], "req_2");
}

#[test]
fn an_error_envelope_from_the_server_fails_the_jump() {
    let refusal = r#"{"id":"req_1","error":{"code":"internal","message":"broken"}}"#;
    let server = start_fake_herdr(vec![refusal.into()], AfterReplies::HangUp);
    let mut directory = connected(&server);
    assert_eq!(
        jump(&mut directory, "dotfiles", "/tmp"),
        Err(JumpError::Server {
            code: "internal".to_string(),
            message: "broken".to_string(),
        })
    );
}

#[test]
fn a_server_that_hangs_up_without_answering_is_a_transport_failure() {
    let server = start_fake_herdr(vec![], AfterReplies::HangUp);
    let mut directory = connected(&server);
    assert!(matches!(
        jump(&mut directory, "dotfiles", "/tmp"),
        Err(JumpError::Transport(_))
    ));
}

#[test]
fn a_missing_socket_is_unreachable_rather_than_a_transport_failure() {
    let absent = std::env::temp_dir().join("hwj-no-such-directory/socket");
    assert!(matches!(
        SocketWorkspaceDirectory::at(&absent, DEADLINE).list(),
        Err(JumpError::Unreachable(_))
    ));
}

#[test]
fn a_server_that_never_answers_fails_on_the_deadline_rather_than_hanging() {
    let server = start_fake_herdr(
        vec![TWO_WORKSPACES.into(), OK.into()],
        AfterReplies::StaySilent,
    );
    let mut directory = SocketWorkspaceDirectory::at(&server.path, Duration::from_millis(150));
    let _answered = (directory.list(), directory.focus("wA"));
    let began = std::time::Instant::now();
    assert!(matches!(
        directory.focus("wA"),
        Err(JumpError::Transport(_))
    ));
    assert!(
        began.elapsed() < Duration::from_millis(300),
        "the 150 ms deadline let a peer that stays silent for 400 ms hold the request"
    );
}

#[test]
fn partial_progress_cannot_extend_the_complete_response_deadline() {
    let server = SocketServer::start(vec![Reply::Chunks(vec![
        (Duration::from_millis(60), "{\"result\":".into()),
        (Duration::from_millis(60), "{\"type\":".into()),
        (Duration::from_millis(60), "\"ok\"}}\n".into()),
    ])]);
    let mut directory = SocketWorkspaceDirectory::at(&server.path, Duration::from_millis(150));
    assert!(matches!(
        directory.focus("wA"),
        Err(JumpError::Transport(_))
    ));
}

#[test]
fn a_blank_socket_acknowledgment_is_not_success() {
    let server = SocketServer::start(vec![Reply::Line(String::new())]);
    let mut directory = connected(&server);
    assert!(matches!(
        directory.focus("wA"),
        Err(JumpError::Malformed(_))
    ));
}
