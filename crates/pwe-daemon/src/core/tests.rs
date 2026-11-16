use super::*;
use pwe_protocol::{DaemonErrorKind, Listens, parse_daemon_reply};

fn core() -> DaemonCore {
    DaemonCore::new(DaemonStatus {
        running: true,
        pid: 7,
        socket: "/run/pwe/daemon.sock".to_string(),
        listens: Listens::ThisMachineOnly,
        started_at: 1,
    })
}

#[test]
fn an_unknown_request_is_refused_and_the_daemon_keeps_serving() {
    let answer = core().answer(r#"{"pwe-daemon":1,"request":"dance"}"#);

    let refusal = parse_daemon_reply(&answer.reply_line).err();
    assert_eq!(
        refusal.map(|refusal| refusal.kind),
        Some(DaemonErrorKind::UnknownRequest)
    );
    assert!(answer.keeps_serving);
}

#[test]
fn stop_is_answered_and_ends_serving() {
    let answer = core().answer(r#"{"pwe-daemon":1,"request":"stop"}"#);

    assert_eq!(
        parse_daemon_reply(&answer.reply_line),
        Ok(json!({ "stopping": true }))
    );
    assert!(!answer.keeps_serving);
}
