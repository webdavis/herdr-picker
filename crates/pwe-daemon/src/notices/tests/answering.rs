use pwe_protocol::{NoticeRequest, NotifyRequest};
use serde_json::json;

use super::*;
use crate::notice_fakes::GONE_PANE;

fn notify(key: &str, level: Level, text: &str, block: &str) -> NoticeRequest {
    NoticeRequest::Notify(NotifyRequest {
        source: "tests".to_string(),
        kind: "run".to_string(),
        key: key.to_string(),
        level,
        text: text.to_string(),
        pane: Some(block.to_string()),
    })
}

#[test]
fn notify_reads_where_the_owner_is_once_and_answers_with_the_new_id() {
    let mut bench = Bench::new("notify", false);

    let answer = bench.desk.answer(
        notify("tests", Level::Finished, "Tests passed", OTHER_PANE),
        NOW,
    );

    assert_eq!(answer, Ok(json!({ "id": format!("{NOW}-1") })));
    assert_eq!(bench.calls.asked.taken(), vec!["where"]);
    let shown = bench.calls.screen.taken();
    assert_eq!(shown.len(), 1, "{shown:?}");
    assert!(
        shown[0].ends_with("finished · tests: Tests passed, in tab api · now"),
        "{shown:?}"
    );
}

#[test]
fn a_notice_about_the_pane_in_view_is_saved_and_stays_silent() {
    let mut bench = Bench::new("in-view", false);

    let answer = bench.desk.answer(
        notify("tests", Level::Finished, "Tests passed", OWNER_PANE),
        NOW,
    );

    assert!(answer.is_ok(), "{answer:?}");
    assert_eq!(bench.calls.screen.taken(), Vec::<String>::new());
    assert_eq!(bench.calls.macos.taken(), Vec::<String>::new());
    let rows = read_inbox(&bench.inbox).unwrap_or_else(|failure| panic!("{failure}"));
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].pane, Some(pane(OWNER_PANE)));
}

#[test]
fn a_pane_rex_does_not_hold_is_left_off_the_notice() {
    let mut bench = Bench::new("gone", false);

    let answer = bench.desk.answer(
        notify("tests", Level::Quiet, "Tests passed", GONE_PANE),
        NOW,
    );

    assert!(answer.is_ok(), "{answer:?}");
    let rows = read_inbox(&bench.inbox).unwrap_or_else(|failure| panic!("{failure}"));
    assert_eq!(
        (rows[0].pane.clone(), rows[0].tab_label.clone()),
        (None, None)
    );
}

#[test]
fn the_same_key_twice_changes_the_notice_in_place() {
    let mut bench = Bench::new("in-place", false);

    let first = bench.desk.answer(
        notify("tests", Level::Finished, "Tests running", OTHER_PANE),
        NOW,
    );
    let second = bench.desk.answer(
        notify("tests", Level::Finished, "Tests failed: 3", OTHER_PANE),
        NOW + 1,
    );

    assert_eq!(first, second);
    let screen = bench.calls.screen.taken();
    assert!(screen[0].starts_with("show float-2"), "{screen:?}");
    assert!(
        screen[1].starts_with("relabel float-2: finished · tests: Tests failed: 3"),
        "{screen:?}"
    );
    assert_eq!(screen.len(), 2, "{screen:?}");
}

#[test]
fn resolve_closes_the_notice_by_its_key() {
    let mut bench = Bench::new("resolve", false);
    let _ = bench.desk.answer(
        notify("agent-1", Level::NeedsYou, "Approve Bash", OTHER_PANE),
        NOW,
    );
    bench.calls.screen.taken();

    let resolved = bench.desk.answer(
        NoticeRequest::Resolve {
            key: "agent-1".to_string(),
        },
        NOW + 1,
    );
    let unknown = bench.desk.answer(
        NoticeRequest::Resolve {
            key: "agent-9".to_string(),
        },
        NOW + 1,
    );

    assert_eq!(resolved, Ok(json!({ "resolved": true })));
    assert_eq!(unknown, Ok(json!({ "resolved": false })));
    assert_eq!(bench.calls.screen.taken(), vec!["close float-2"]);
}
