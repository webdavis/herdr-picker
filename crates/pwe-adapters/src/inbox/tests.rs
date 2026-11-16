use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use pwe_domain::{InboxLimits, Level, Notice, Pane};

use super::*;

const DAY: u64 = 86_400;

struct TempInbox(PathBuf);

impl TempInbox {
    fn new(name: &str) -> Self {
        let directory =
            std::env::temp_dir().join(format!("pwe-inbox-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        Self(directory)
    }

    fn file(&self) -> PathBuf {
        self.0.join("state/inbox.jsonl")
    }
}

impl Drop for TempInbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn row(id: &str, updated_at: u64, read_at: Option<u64>) -> Notice {
    Notice {
        id: id.to_string(),
        key: format!("key-{id}"),
        source: "cargo".to_string(),
        kind: "tests".to_string(),
        level: Level::NeedsYou,
        text: "Tests failed: 3".to_string(),
        pane: Some(Pane {
            rex_session: "work".to_string(),
            tab: "t1".to_string(),
            block: "b1".to_string(),
        }),
        tab_label: Some("api".to_string()),
        raised_at: updated_at,
        updated_at,
        resolved_at: None,
        read_at,
    }
}

#[test]
fn every_notice_is_kept_as_one_private_row_with_kebab_case_fields() {
    let inbox = TempInbox::new("kept");

    save_inbox_row(&inbox.file(), &row("1", 100, None)).expect("saved");
    save_inbox_row(&inbox.file(), &row("2", 101, Some(105))).expect("saved");

    let text = fs::read_to_string(inbox.file()).expect("inbox");
    assert_eq!(
        text.lines().next(),
        Some(
            r#"{"id":"1","key":"key-1","source":"cargo","kind":"tests","level":"needs-you","text":"Tests failed: 3","pane":{"rex-session":"work","tab":"t1","block":"b1"},"tab-label":"api","raised-at":100,"updated-at":100}"#
        )
    );
    assert_eq!(text.lines().count(), 2);
    let mode = fs::metadata(inbox.file())
        .expect("metadata")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o600);
    assert_eq!(
        read_inbox(&inbox.file()).expect("read"),
        vec![row("1", 100, None), row("2", 101, Some(105))]
    );
}

#[test]
fn saving_a_row_again_changes_it_in_place() {
    let inbox = TempInbox::new("in-place");
    save_inbox_row(&inbox.file(), &row("1", 100, None)).expect("saved");
    save_inbox_row(&inbox.file(), &row("2", 101, None)).expect("saved");

    save_inbox_row(&inbox.file(), &row("1", 100, Some(110))).expect("saved");

    assert_eq!(
        read_inbox(&inbox.file()).expect("read"),
        vec![row("1", 100, Some(110)), row("2", 101, None)]
    );
}

#[test]
fn a_missing_file_is_an_empty_inbox() {
    let inbox = TempInbox::new("missing");

    assert_eq!(read_inbox(&inbox.file()).expect("read"), Vec::new());
}

#[test]
fn a_broken_row_is_named_by_its_line() {
    let inbox = TempInbox::new("broken");
    save_inbox_row(&inbox.file(), &row("1", 100, None)).expect("saved");
    let mut text = fs::read_to_string(inbox.file()).expect("inbox");
    text.push_str("not json\n");
    fs::write(inbox.file(), text).expect("broken");

    let refusal = read_inbox(&inbox.file()).expect_err("refused").to_string();

    assert!(refusal.contains("line 2"), "{refusal}");
}

#[test]
fn read_rows_older_than_keep_days_are_pruned_and_unread_rows_stay() {
    let inbox = TempInbox::new("pruned");
    let now = 100 * DAY;
    for saved in [
        row("old-read", now - 40 * DAY, Some(now - 40 * DAY)),
        row("old-unread", now - 40 * DAY, None),
        row("new-read", now - DAY, Some(now - DAY)),
    ] {
        save_inbox_row(&inbox.file(), &saved).expect("saved");
    }

    prune_inbox_file(&inbox.file(), now, &InboxLimits::default()).expect("pruned");

    let ids: Vec<String> = read_inbox(&inbox.file())
        .expect("read")
        .into_iter()
        .map(|row| row.id)
        .collect();
    assert_eq!(ids, vec!["old-unread", "new-read"]);
}
