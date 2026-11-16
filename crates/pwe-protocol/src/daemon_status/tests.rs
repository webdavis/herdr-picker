use super::*;

fn status() -> DaemonStatus {
    DaemonStatus {
        running: true,
        pid: 4242,
        socket: "/run/user/501/pwe/daemon.sock".to_string(),
        listens: Listens::ThisMachineOnly,
        started_at: 1_700_000_000,
        rex: RexLink::Reconnecting,
    }
}

#[test]
fn the_status_result_uses_kebab_case_words() {
    assert_eq!(
        daemon_status_result(&status()),
        json!({
            "running": true,
            "pid": 4242,
            "socket": "/run/user/501/pwe/daemon.sock",
            "listens": "this-machine-only",
            "started-at": 1_700_000_000,
            "rex": "reconnecting",
        })
    );
}

#[test]
fn a_status_result_reads_back_as_the_same_status() {
    assert_eq!(
        parse_daemon_status(&daemon_status_result(&status())),
        Ok(status())
    );
}

#[test]
fn a_status_missing_a_field_is_unreadable() {
    let read = parse_daemon_status(&json!({"running": true}));

    assert!(matches!(
        read,
        Err(DaemonError {
            kind: DaemonErrorKind::UnreadableReply,
            ..
        })
    ));
}
