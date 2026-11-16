use std::fs;
use std::path::PathBuf;

use pwe_adapters::save_inbox_row;
use pwe_domain::{Level, Notice};

use super::*;

struct TempInbox(PathBuf);

impl TempInbox {
    fn new(name: &str) -> Self {
        let directory =
            std::env::temp_dir().join(format!("pwe-cli-inbox-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        Self(directory)
    }

    fn file(&self) -> PathBuf {
        self.0.join("inbox.jsonl")
    }

    fn holding(name: &str, rows: &[Notice]) -> Self {
        let inbox = Self::new(name);
        for row in rows {
            save_inbox_row(&inbox.file(), row).expect("saved");
        }
        inbox
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
        key: id.to_string(),
        source: "cargo".to_string(),
        kind: "tests".to_string(),
        level: Level::Finished,
        text: "Tests passed".to_string(),
        pane: None,
        tab_label: None,
        raised_at: updated_at,
        updated_at,
        resolved_at: None,
        read_at,
    }
}

fn listed_ids(json: &str) -> Vec<String> {
    let listing: serde_json::Value = serde_json::from_str(json).expect("JSON");
    listing["inbox"]
        .as_array()
        .expect("an inbox list")
        .iter()
        .map(|row| row["id"].as_str().expect("an id").to_string())
        .collect()
}

#[test]
fn pwe_inbox_prints_every_row_newest_first_as_json() {
    let inbox = TempInbox::holding(
        "every",
        &[row("old", 100, Some(120)), row("new", 200, None)],
    );

    let json = listing(&inbox.file(), &[]).expect("listed");

    assert_eq!(listed_ids(&json), vec!["new", "old"]);
    assert!(json.starts_with(r#"{"inbox":[{"id":"new","key":"new","source":"cargo","kind":"tests","level":"finished","text":"Tests passed","raised-at":200,"updated-at":200}"#), "{json}");
}

#[test]
fn pwe_inbox_unread_leaves_the_read_rows_out() {
    let inbox = TempInbox::holding(
        "unread",
        &[row("read", 100, Some(120)), row("unread", 90, None)],
    );

    let json = listing(&inbox.file(), &["--unread"]).expect("listed");

    assert_eq!(listed_ids(&json), vec!["unread"]);
}

#[test]
fn a_missing_inbox_file_is_an_empty_inbox() {
    let inbox = TempInbox::new("missing");

    assert_eq!(
        listing(&inbox.file(), &[]).expect("listed"),
        r#"{"inbox":[]}"#
    );
}

#[test]
fn an_unknown_option_is_a_usage_error() {
    let inbox = TempInbox::new("unknown");

    assert!(matches!(
        listing(&inbox.file(), &["--all"]),
        Err(CommandError::Usage)
    ));
}
