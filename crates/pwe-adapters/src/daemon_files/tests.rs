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
fn the_log_sits_in_the_state_folder() {
    assert_eq!(
        daemon_log_file(None, Some("/h")),
        PathBuf::from("/h/.local/state/pwe/daemon.log")
    );
}

#[test]
fn the_launch_agent_and_the_unit_are_named_pwe_daemon() {
    assert_eq!(
        launch_agent_file(Some("/h")),
        PathBuf::from("/h/Library/LaunchAgents/pwe-daemon.plist")
    );
    assert_eq!(
        systemd_unit_file(None, Some("/h")),
        PathBuf::from("/h/.config/systemd/user/pwe-daemon.service")
    );
    assert_eq!(
        systemd_unit_file(Some("/c"), Some("/h")),
        PathBuf::from("/c/systemd/user/pwe-daemon.service")
    );
}
