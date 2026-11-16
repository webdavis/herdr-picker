use super::*;

#[test]
fn the_socket_sits_in_the_runtime_folder_when_one_is_set() {
    assert_eq!(
        daemon_socket_file(Some("/run/user/501"), Some("/s"), Some("/h")),
        PathBuf::from("/run/user/501/pwe/daemon.sock")
    );
}

#[test]
fn without_a_runtime_folder_the_socket_sits_in_the_state_folder() {
    assert_eq!(
        daemon_socket_file(Some(""), Some("/s"), Some("/h")),
        PathBuf::from("/s/pwe/daemon.sock")
    );
    assert_eq!(
        daemon_socket_file(None, None, Some("/h")),
        PathBuf::from("/h/.local/state/pwe/daemon.sock")
    );
}

#[test]
fn asking_where_no_daemon_listens_says_it_is_not_running() {
    let socket = std::env::temp_dir().join(format!("pwe-absent-{}.sock", std::process::id()));

    let asked = ask_daemon(&socket, DaemonRequest::Status);

    assert!(
        matches!(asked, Err(DaemonAskError::NotRunning)),
        "{asked:?}"
    );
}
