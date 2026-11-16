mod support;

use serde_json::json;
use support::{TempHome, json_of, succeeded};

#[test]
fn start_then_status_says_running_and_not_enabled() {
    let home = TempHome::new("start").with_no_rex_app();

    succeeded(home.pwe(&["daemon", "start"]));
    let status = json_of(&succeeded(
        home.pwe(&["daemon", "status", "--format", "json"]),
    ));

    assert_eq!(status["running"], json!(true));
    assert_eq!(status["enabled"], json!(false));
    assert_eq!(status["listens"], json!("this-machine-only"));
    assert!(status["pid"].is_u64(), "{status}");
}

#[test]
fn stop_then_status_says_not_running_in_plain_words() {
    let home = TempHome::new("stop").with_no_rex_app();
    succeeded(home.pwe(&["daemon", "start"]));

    succeeded(home.pwe(&["daemon", "stop"]));
    let status = succeeded(home.pwe(&["daemon", "status"]));

    assert!(status.contains("not running"), "{status}");
    assert!(status.contains("not enabled"), "{status}");
    assert!(status.contains("this machine only"), "{status}");
}

#[test]
fn starting_twice_keeps_the_one_daemon() {
    let home = TempHome::new("twice").with_no_rex_app();
    succeeded(home.pwe(&["daemon", "start"]));
    let first = json_of(&succeeded(
        home.pwe(&["daemon", "status", "--format", "json"]),
    ));

    let again = succeeded(home.pwe(&["daemon", "start"]));
    let second = json_of(&succeeded(
        home.pwe(&["daemon", "status", "--format", "json"]),
    ));

    assert!(again.contains("already running"), "{again}");
    assert_eq!(first["pid"], second["pid"]);
}

#[test]
fn status_with_the_old_output_option_is_a_usage_error() {
    let home = TempHome::new("old-output").with_no_rex_app();

    let output = home.pwe(&["daemon", "status", "--output", "json"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("pwe daemon status [--format json|text]"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn status_with_format_text_prints_plain_words() {
    let home = TempHome::new("format-text").with_no_rex_app();

    let status = succeeded(home.pwe(&["daemon", "status", "--format", "text"]));

    assert!(status.contains("not running"), "{status}");
}

#[test]
fn status_says_rex_is_absent_on_a_machine_without_rex() {
    let home = TempHome::new("rex-absent").with_no_rex_app();
    succeeded(home.pwe(&["daemon", "start"]));

    let status = json_of(&succeeded(
        home.pwe(&["daemon", "status", "--format", "json"]),
    ));
    let words = succeeded(home.pwe(&["daemon", "status"]));

    assert_eq!(status["rex"], json!("absent"));
    assert!(words.contains("Rex is not on this machine."), "{words}");
}

#[test]
fn status_says_rex_is_connected_while_its_events_flow() {
    let home = TempHome::new("rex-connected").with_no_rex_app().with_fake_program(
        "rex",
        "#!/bin/sh\nprintf '%s\\n' '{\"type\":\"session_destroyed\",\"payload\":{\"session_id\":\"session:00000000-0000-7000-8000-00000000a009\"}}'\nexec sleep 30\n",
    );
    succeeded(home.pwe(&["daemon", "start"]));

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let mut status = json_of(&succeeded(
        home.pwe(&["daemon", "status", "--format", "json"]),
    ));
    while status["rex"] != json!("connected") && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(100));
        status = json_of(&succeeded(
            home.pwe(&["daemon", "status", "--format", "json"]),
        ));
    }

    assert_eq!(status["rex"], json!("connected"));
}
