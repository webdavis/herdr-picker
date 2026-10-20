use herdr_picker_domain::{Action, PickerRow, Preview};

use super::*;
use crate::builtin::rows::key;

const BREW: &str = r#"[
  {"name": "atuin", "status": "none", "user": null, "file": "/nowhere/sh.brew.atuin.plist", "exit_code": null},
  {"name": "moshi-hook", "status": "started", "user": "stephen", "file": "/nowhere/moshi.plist", "pid": 4321, "exit_code": 0}
]"#;

const LAUNCHCTL: &str = "PID\tStatus\tLabel
726\t0\tcom.apple.trustd.agent
-\t78\tcom.example.failing
";

#[test]
fn a_running_brew_service_shows_green_and_prepares_its_info_command() {
    let rows = brew_rows(BREW).expect("brew rows");

    assert_eq!(
        rows[1],
        PickerRow {
            label: "moshi-hook  \u{1b}[38;2;166;227;161mstarted\u{1b}[0m".to_string(),
            preview: Some(Preview::Text(
                "brew services info moshi-hook\n\nStatus: started\nRuns as: stephen\nService file: /nowhere/moshi.plist\nPID: 4321\nLast exit code: 0\nEnter prepares this command.".to_string()
            )),
            enter: Action::Insert("brew services info moshi-hook".to_string()),
            keys: vec![(key("ctrl-enter"), Action::Run("brew services info moshi-hook".to_string()))],
            group: Some("Homebrew".to_string()),
        }
    );
    assert_eq!(rows[0].label, "atuin  \u{1b}[2mnone\u{1b}[0m");
}

#[test]
fn a_brew_services_log_files_come_from_its_plist_for_alt_d() {
    let plist = "<plist><dict>\n\t<key>StandardOutPath</key>\n\t<string>/var/log/x.log</string>\n\t<key>StandardErrorPath</key>\n\t<string>/var/log/x.log</string>\n</dict></plist>";

    assert_eq!(log_paths(plist), vec!["/var/log/x.log".to_string()]);
}

#[test]
fn a_launchd_job_shows_its_pid_or_exit_status_and_alt_d_reads_a_running_ones_log() {
    let rows = launchd_rows(LAUNCHCTL);

    assert_eq!(rows.len(), 2);
    assert_eq!(
        rows[0].label,
        "com.apple.trustd.agent  \u{1b}[38;2;166;227;161mpid 726\u{1b}[0m"
    );
    assert_eq!(
        rows[0].keys[1],
        (
            key("alt-d"),
            Action::Insert(
                "/usr/bin/log show --last 1h --predicate 'processIdentifier == 726'".to_string()
            )
        )
    );
    assert_eq!(
        rows[1].label,
        "com.example.failing  \u{1b}[38;2;243;139;168mexit 78\u{1b}[0m"
    );
    assert_eq!(rows[1].keys.len(), 1);
    assert_eq!(rows[1].group.as_deref(), Some("launchd"));
}

#[test]
fn a_status_that_is_neither_running_nor_failed_is_peach() {
    assert_eq!(
        status_line("x", "scheduled"),
        "x  \u{1b}[38;2;250;179;135mscheduled\u{1b}[0m"
    );
    assert_eq!(status_line("x", ""), "x");
}
