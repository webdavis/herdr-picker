use super::*;

#[test]
fn asking_where_no_daemon_listens_says_it_is_not_running() {
    let socket = std::env::temp_dir().join(format!("pwe-absent-{}.sock", std::process::id()));

    let asked = ask_daemon(&socket, DaemonRequest::Status);

    assert!(
        matches!(asked, Err(DaemonAskError::NotRunning)),
        "{asked:?}"
    );
}
