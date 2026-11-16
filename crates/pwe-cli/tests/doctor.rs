mod support;

use serde_json::json;
use support::{TempHome, json_of, succeeded};

const PERMISSION_REFUSED: &str = "#!/bin/sh\n/bin/cat >/dev/null\necho 'execution error: Error: Not authorized to send Apple events to Rex Beta. (-1743)' >&2\nexit 1\n";

const REX_IN_FRONT: &str = "#!/bin/sh\n/bin/cat >/dev/null\necho '{\"running\":true,\"frontmost\":true,\"session\":{\"id\":\"host:example/session:example\",\"name\":\"example\",\"remote\":false}}'\n";

fn automation_check(report: &serde_json::Value) -> &serde_json::Value {
    report["checks"]
        .as_array()
        .and_then(|checks| {
            checks
                .iter()
                .find(|check| check["name"] == "automation-permission")
        })
        .unwrap_or_else(|| panic!("no automation-permission check in {report}"))
}

#[test]
fn a_refused_permission_names_where_to_grant_it() {
    let home = TempHome::new("doctor-refused").with_fake_program("osascript", PERMISSION_REFUSED);

    let output = home.pwe(&["doctor", "--format", "json"]);

    assert_eq!(output.status.code(), Some(1));
    let report = json_of(&String::from_utf8_lossy(&output.stdout));
    let check = automation_check(&report);
    assert_eq!(check["result"], json!("failed"));
    let what_to_do = check["what-to-do"].as_str().unwrap_or_default();
    assert!(
        [
            "System Settings",
            "Privacy & Security",
            "Automation",
            "Rex Beta"
        ]
        .iter()
        .all(|place| what_to_do.contains(place)),
        "{what_to_do}"
    );
}

#[test]
fn a_granted_permission_passes_and_exits_zero() {
    let home = TempHome::new("doctor-granted").with_fake_program("osascript", REX_IN_FRONT);

    let report = json_of(&succeeded(home.pwe(&["doctor", "--format", "json"])));

    let check = automation_check(&report);
    assert_eq!(
        (&check["result"], &check["what-to-do"]),
        (&json!("passed"), &json!(null))
    );
}

#[test]
fn the_plain_report_says_the_same_in_words() {
    let home = TempHome::new("doctor-words").with_fake_program("osascript", PERMISSION_REFUSED);

    let output = home.pwe(&["doctor"]);

    assert_eq!(output.status.code(), Some(1));
    let words = String::from_utf8_lossy(&output.stdout);
    assert!(
        words.contains("failed")
            && words.contains("automation-permission")
            && words.contains("Privacy & Security"),
        "{words}"
    );
}

#[test]
fn a_closed_rex_fails_the_check_and_says_to_open_it() {
    let home = TempHome::new("doctor-closed").with_fake_program(
        "osascript",
        "#!/bin/sh\n/bin/cat >/dev/null\necho '{\"running\":false}'\n",
    );

    let output = home.pwe(&["doctor", "--format", "json"]);

    assert_eq!(output.status.code(), Some(1));
    let report = json_of(&String::from_utf8_lossy(&output.stdout));
    let what_to_do = automation_check(&report)["what-to-do"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    assert!(what_to_do.contains("Open Rex Beta"), "{what_to_do}");
}

#[test]
fn the_rex_app_name_comes_from_config() {
    let home = TempHome::new("doctor-app-name").with_fake_program(
        "osascript",
        "#!/bin/sh\n/bin/cat >/dev/null\nprintf '%s\\n' \"$@\" > \"$HOME/osascript-arguments\"\necho '{\"running\":false}'\n",
    );
    let config = home.folder.join(".config/pwe");
    std::fs::create_dir_all(&config).unwrap_or_else(|failure| panic!("{failure}"));
    std::fs::write(
        config.join("config.toml"),
        "[rex]\napp-name = \"Example Terminal\"\n",
    )
    .unwrap_or_else(|failure| panic!("{failure}"));

    let output = home.pwe(&["doctor", "--format", "json"]);

    let arguments =
        std::fs::read_to_string(home.folder.join("osascript-arguments")).unwrap_or_default();
    assert_eq!(
        arguments.lines().last(),
        Some("Example Terminal"),
        "{arguments}"
    );
    let report = json_of(&String::from_utf8_lossy(&output.stdout));
    let what_to_do = automation_check(&report)["what-to-do"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    assert!(what_to_do.contains("Open Example Terminal"), "{what_to_do}");
}
