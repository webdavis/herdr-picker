use pwe_protocol::Refusal;

use super::{hand, method_and_params, ok, pane, screen};
use crate::pane_handover::HandoverError;
use crate::socket_server::SocketServer;

#[test]
fn a_folder_over_a_shell_with_an_empty_prompt_is_changed_into() {
    let server = SocketServer::start(vec![
        pane("w1:p2", None, "unknown"),
        screen("~/repo\n❯ \n"),
        ok(),
        ok(),
    ]);

    assert_eq!(hand(&server, "cd\tsrc"), Ok(()));
    let _ = (server.seen(), server.seen());
    assert_eq!(
        method_and_params(&server),
        (
            "pane.send_text".to_string(),
            serde_json::json!({"pane_id": "w1:p2", "text": "cd src"})
        )
    );
    assert_eq!(method_and_params(&server).0, "pane.send_keys");
}

#[test]
fn a_shell_draft_refuses_a_folder_or_run_but_takes_an_insert_after_it() {
    let drafted = || {
        SocketServer::start(vec![
            pane("w1:p2", None, "unknown"),
            screen("~/repo\n❯ git sta\n"),
            ok(),
        ])
    };
    let refused = Err(HandoverError::Refused(Refusal::Draft(
        "git sta".to_string(),
    )));

    assert_eq!(hand(&drafted(), "cd\tsrc"), refused);
    assert_eq!(hand(&drafted(), "run\tjust test"), refused);
    let server = SocketServer::start(vec![pane("w1:p2", None, "unknown"), ok()]);
    assert_eq!(hand(&server, "insert\tsrc/main.rs"), Ok(()));
    let _ = server.seen();
    assert_eq!(
        method_and_params(&server),
        (
            "pane.send_text".to_string(),
            serde_json::json!({"pane_id": "w1:p2", "text": "src/main.rs"})
        )
    );
}

#[test]
fn an_instruction_pwe_cannot_read_is_refused_before_herdr_is_asked() {
    let server = SocketServer::start(vec![]);

    assert_eq!(
        hand(&server, "copy\tx"),
        Err(HandoverError::Unreadable("copy\tx".to_string()))
    );
}
