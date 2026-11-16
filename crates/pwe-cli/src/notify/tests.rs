use std::fs;
use std::path::PathBuf;

use pwe_adapters::read_inbox;
use pwe_domain::Level;
use pwe_protocol::{NoticeRequest, NotifyRequest};

use super::*;

const NOW: u64 = 1_790_000_100;
const PID: u32 = 4242;

struct TempState(PathBuf);

impl TempState {
    fn new(name: &str) -> Self {
        let folder = std::env::temp_dir().join(format!("pwe-notify-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&folder);
        Self(folder)
    }

    fn sent(&self, options: &[&str]) -> Sent {
        let request = request(options, NOW, PID).unwrap_or_else(|_| panic!("{options:?}"));
        send(&self.0.join("daemon.sock"), &self.inbox(), &request, NOW)
            .unwrap_or_else(|_| panic!("{options:?} was not sent"))
    }

    fn inbox(&self) -> PathBuf {
        self.0.join("inbox.jsonl")
    }
}

impl Drop for TempState {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn notify_flags_become_the_daemons_fields() {
    let options = [
        "--level",
        "finished",
        "--source",
        "tests",
        "--kind",
        "run",
        "--key",
        "tests-api",
        "--pane",
        "block:1",
        "Tests passed",
    ];

    assert_eq!(
        request(&options, NOW, PID).ok(),
        Some(NoticeRequest::Notify(NotifyRequest {
            source: "tests".to_string(),
            kind: "run".to_string(),
            key: "tests-api".to_string(),
            level: Level::Finished,
            text: "Tests passed".to_string(),
            pane: Some("block:1".to_string()),
        }))
    );
}

#[test]
fn without_a_key_or_kind_each_notify_is_its_own_notice() {
    let options = ["--source", "tests", "--level", "quiet", "Tests passed"];

    let Ok(NoticeRequest::Notify(notify)) = request(&options, NOW, PID) else {
        panic!("{options:?} was refused");
    };

    assert_eq!(
        (notify.key.as_str(), notify.kind.as_str(), notify.pane),
        ("tests-1790000100-4242", "notice", None)
    );
}

#[test]
fn an_unknown_level_names_the_accepted_words() {
    let refusal = request(&["--source", "t", "--level", "loud", "x"], NOW, PID);

    assert!(
        matches!(&refusal, Err(CommandError::Failed(reason)) if reason.contains("needs-you, finished, quiet")),
        "{refusal:?}"
    );
}

#[test]
fn a_notify_without_its_text_or_source_is_a_usage_error() {
    for options in [
        &["--source", "tests", "--level", "quiet"][..],
        &["--level", "quiet", "Tests passed"][..],
        &["--source", "tests", "--level", "quiet", "one", "two"][..],
        &[
            "--source", "tests", "--level", "quiet", "--colour", "red", "x",
        ][..],
    ] {
        assert!(
            matches!(request(options, NOW, PID), Err(CommandError::Usage)),
            "{options:?}"
        );
    }
}

#[test]
fn resolve_takes_its_key() {
    assert_eq!(
        request(&["resolve", "--key", "tests-api"], NOW, PID).ok(),
        Some(NoticeRequest::Resolve {
            key: "tests-api".to_string()
        })
    );
}

#[test]
fn with_the_daemon_stopped_the_row_is_saved_and_the_owner_is_told_to_start_it() {
    let state = TempState::new("stopped");

    let sent = state.sent(&[
        "--source",
        "te\u{1b}sts",
        "--level",
        "finished",
        "Tests\npassed",
    ]);

    assert!(sent.daemon_stopped);
    assert!(
        sent.printed.starts_with(r#"{"id":"1790000100-cli-"#),
        "{}",
        sent.printed
    );
    let rows = read_inbox(&state.inbox()).unwrap_or_else(|failure| panic!("{failure}"));
    assert_eq!(rows.len(), 1);
    assert_eq!(
        (rows[0].source.as_str(), rows[0].text.as_str()),
        ("tests", "Testspassed")
    );
    assert!(DAEMON_STOPPED.contains("pwe daemon start"));
}

#[test]
fn with_the_daemon_stopped_resolve_marks_the_saved_row_resolved() {
    let state = TempState::new("resolve");
    state.sent(&[
        "--source",
        "tests",
        "--level",
        "needs-you",
        "--key",
        "k",
        "Approve",
    ]);

    let sent = state.sent(&["resolve", "--key", "k"]);

    assert_eq!(sent.printed, r#"{"resolved":true}"#);
    let rows = read_inbox(&state.inbox()).unwrap_or_else(|failure| panic!("{failure}"));
    assert_eq!(rows[0].resolved_at, Some(NOW));
}
