use std::fs;
use std::time::{Duration, SystemTime};

use herdr_picker_domain::{Action, Preview};

use super::*;
use crate::builtin::tests::Tree;

const EVENT: &str = r#"{"eventType":"logEvent","subsystem":"com.apple.wifi","processImagePath":"\/usr\/sbin\/wifid","timestamp":"2026-10-19 06:07:44.998484-0600","eventMessage":"link up"}"#;

#[test]
fn a_log_event_shows_its_time_process_and_message_and_enter_copies_the_message() {
    let row = log_row(EVENT).expect("a row");

    assert_eq!(
        row.label,
        "\u{1b}[2m2026-10-19 06:07:44.998484-0600\u{1b}[0m \u{1b}[38;2;148;226;213mwifid\u{1b}[0m link up"
    );
    assert_eq!(row.enter, Action::Copy("link up".to_string()));
    let Some(Preview::Text(preview)) = row.preview else {
        panic!("a text preview")
    };
    assert!(
        preview.starts_with("wifid\n\nTime       2026-10-19 06:07:44.998484-0600\nProcess    /usr/sbin/wifid\nType       logEvent\nSubsystem  com.apple.wifi\n\nFull log entry\n{\n"),
        "{preview}"
    );
}

#[test]
fn a_line_that_is_not_an_event_is_skipped() {
    assert_eq!(log_row("Filtering the log data using \"x\""), None);
    assert_eq!(log_row("[]"), None);
}

#[test]
fn crash_reports_list_newest_first_and_enter_prepares_console() {
    let tree = Tree::new("crashes", &["Retired/old.ips", "new.crash", "notes.txt"]);
    let old = SystemTime::UNIX_EPOCH + Duration::from_secs(1_792_404_464);
    fs::File::options()
        .write(true)
        .open(tree.path("Retired/old.ips"))
        .expect("a report")
        .set_modified(old)
        .expect("a time");

    let rows = crash_rows(&[tree.0.clone(), tree.0.join("missing")], 0);

    assert_eq!(rows.len(), 2);
    assert!(
        rows[0].label.starts_with("new.crash  "),
        "{}",
        rows[0].label
    );
    assert_eq!(rows[1].label, "old.ips  \u{1b}[2m2026-10-19 10:07\u{1b}[0m");
    let open = format!("open -a Console {}", tree.path("Retired/old.ips"));
    assert_eq!(rows[1].enter, Action::Insert(open));
    assert_eq!(
        rows[1].preview,
        Some(Preview::Path(tree.path("Retired/old.ips")))
    );
}
