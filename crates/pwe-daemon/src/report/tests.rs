use super::*;
use pwe_protocol::{Listens, RexLink};

fn report(status: Option<DaemonStatus>, enabled: bool) -> DaemonReport {
    DaemonReport {
        status,
        enabled,
        socket: PathBuf::from("/s/pwe/daemon.sock"),
    }
}

fn running() -> Option<DaemonStatus> {
    Some(DaemonStatus {
        running: true,
        pid: 42,
        socket: "/s/pwe/daemon.sock".to_string(),
        listens: Listens::ThisMachineOnly,
        started_at: 1_700_000_000,
        rex: RexLink::Connected,
    })
}

#[test]
fn a_running_enabled_daemon_reports_its_pid_and_start() {
    assert_eq!(
        report(running(), true).json(),
        json!({
            "running": true,
            "enabled": true,
            "socket": "/s/pwe/daemon.sock",
            "listens": "this-machine-only",
            "pid": 42,
            "started-at": 1_700_000_000,
            "rex": "connected",
        })
    );
}

#[test]
fn a_stopped_daemon_reports_no_pid() {
    let json = report(None, false).json();

    assert_eq!(json["running"], json!(false));
    assert!(json.get("pid").is_none(), "{json}");
}

#[test]
fn the_plain_words_say_the_same_facts() {
    let words = report(running(), true).plain_words();

    assert!(words.contains("running as process 42"), "{words}");
    assert!(words.contains("is enabled"), "{words}");
    assert!(words.contains("this machine only"), "{words}");
}

#[test]
fn the_plain_words_say_how_rex_is_followed() {
    let mut status = running();
    let mut words_for = |link| {
        if let Some(status) = status.as_mut() {
            status.rex = link;
        }
        report(status.clone(), false).plain_words()
    };

    let connected = words_for(RexLink::Connected);
    let reconnecting = words_for(RexLink::Reconnecting);
    let absent = words_for(RexLink::Absent);

    assert!(
        connected.contains("It follows Rex's events."),
        "{connected}"
    );
    assert!(
        reconnecting.contains("It waits for Rex's events to open again."),
        "{reconnecting}"
    );
    assert!(absent.contains("Rex is not on this machine."), "{absent}");
}
