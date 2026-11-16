mod support;

use std::fs;
use std::process::Command;

use serde_json::json;
use support::{TempHome, json_of, succeeded};

#[cfg(target_os = "macos")]
fn user_id() -> String {
    let output = Command::new("id")
        .arg("-u")
        .output()
        .unwrap_or_else(|failure| panic!("{failure}"));
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

#[cfg(target_os = "macos")]
#[test]
fn enable_writes_the_job_and_bootstraps_it_for_this_user() {
    let home = TempHome::new("enable")
        .with_no_rex_app()
        .with_fake_launchctl_and_systemctl();

    succeeded(home.pwe(&["daemon", "enable"]));

    let job = home.folder.join("Library/LaunchAgents/pwe-daemon.plist");
    let plist = fs::read_to_string(&job).unwrap_or_else(|failure| panic!("{failure}"));
    assert!(plist.contains(env!("CARGO_BIN_EXE_pwe")), "{plist}");
    assert!(plist.contains("/fake-bin:/usr/bin:/bin"), "{plist}");
    assert!(
        home.system_calls()
            .contains(&format!("bootstrap gui/{} {}", user_id(), job.display())),
        "{:?}",
        home.system_calls()
    );
    let status = json_of(&succeeded(
        home.pwe(&["daemon", "status", "--format", "json"]),
    ));
    assert_eq!(
        (&status["running"], &status["enabled"]),
        (&json!(true), &json!(true))
    );
}

#[cfg(target_os = "macos")]
#[test]
fn disable_boots_the_job_out_and_removes_it() {
    let home = TempHome::new("disable")
        .with_no_rex_app()
        .with_fake_launchctl_and_systemctl();
    succeeded(home.pwe(&["daemon", "enable"]));

    succeeded(home.pwe(&["daemon", "disable"]));

    assert!(
        !home
            .folder
            .join("Library/LaunchAgents/pwe-daemon.plist")
            .exists()
    );
    assert!(
        home.system_calls()
            .contains(&format!("bootout gui/{}/pwe-daemon", user_id())),
        "{:?}",
        home.system_calls()
    );
    let status = json_of(&succeeded(
        home.pwe(&["daemon", "status", "--format", "json"]),
    ));
    assert_eq!(
        (&status["running"], &status["enabled"]),
        (&json!(false), &json!(false))
    );
}

#[cfg(target_os = "linux")]
#[test]
fn enable_writes_the_unit_and_enables_it_for_this_user() {
    let home = TempHome::new("enable")
        .with_no_rex_app()
        .with_fake_launchctl_and_systemctl();

    succeeded(home.pwe(&["daemon", "enable"]));

    let unit = fs::read_to_string(home.folder.join(".config/systemd/user/pwe-daemon.service"))
        .unwrap_or_else(|failure| panic!("{failure}"));
    assert!(unit.contains(env!("CARGO_BIN_EXE_pwe")), "{unit}");
    assert_eq!(
        home.system_calls(),
        vec![
            "--user daemon-reload",
            "--user enable --now pwe-daemon.service"
        ]
    );
    let status = json_of(&succeeded(
        home.pwe(&["daemon", "status", "--format", "json"]),
    ));
    assert_eq!(
        (&status["running"], &status["enabled"]),
        (&json!(true), &json!(true))
    );
}

#[cfg(target_os = "linux")]
#[test]
fn disable_stops_the_unit_and_removes_it() {
    let home = TempHome::new("disable")
        .with_no_rex_app()
        .with_fake_launchctl_and_systemctl();
    succeeded(home.pwe(&["daemon", "enable"]));

    succeeded(home.pwe(&["daemon", "disable"]));

    assert!(
        !home
            .folder
            .join(".config/systemd/user/pwe-daemon.service")
            .exists()
    );
    assert!(
        home.system_calls()
            .contains(&"--user disable --now pwe-daemon.service".to_string()),
        "{:?}",
        home.system_calls()
    );
    let status = json_of(&succeeded(
        home.pwe(&["daemon", "status", "--format", "json"]),
    ));
    assert_eq!(
        (&status["running"], &status["enabled"]),
        (&json!(false), &json!(false))
    );
}

#[test]
fn a_missing_system_program_is_named() {
    let home = TempHome::new("missing").with_no_rex_app();
    let empty_bin = home.folder.join("empty-bin");
    fs::create_dir_all(&empty_bin).unwrap_or_else(|failure| panic!("{failure}"));

    let output = Command::new(env!("CARGO_BIN_EXE_pwe"))
        .args(["daemon", "enable"])
        .env("HOME", &home.folder)
        .env("PATH", &empty_bin)
        .env_remove("XDG_RUNTIME_DIR")
        .env_remove("XDG_STATE_HOME")
        .env_remove("XDG_CONFIG_HOME")
        .output()
        .unwrap_or_else(|failure| panic!("{failure}"));

    let refusal = String::from_utf8_lossy(&output.stderr);
    let program = if cfg!(target_os = "macos") {
        "launchctl"
    } else {
        "systemctl"
    };
    assert!(!output.status.success());
    assert!(
        refusal.contains(program) && refusal.contains("not found"),
        "{refusal}"
    );
}
