use pwe_adapters::save_inbox_row;
use pwe_protocol::NoticeRequest;
use serde_json::json;

use super::*;

fn raise_waiting(bench: &mut Bench, key: &str) {
    bench.raise(key, Level::NeedsYou, NOW);
}

#[test]
fn waiting_lists_what_needs_the_owner_as_inbox_rows() {
    let mut bench = Bench::new("waiting", false);
    raise_waiting(&mut bench, "agent-1");

    let answer = bench.desk.answer(NoticeRequest::Waiting, NOW + 1);

    let waiting = answer.unwrap_or_else(|refusal| panic!("{refusal:?}"));
    assert_eq!(waiting["waiting"][0]["id"], json!(format!("{NOW}-1")));
    assert_eq!(waiting["waiting"][0]["level"], json!("needs-you"));
    assert_eq!(waiting["waiting"][0]["tab-label"], json!("api"));
}

#[test]
fn hide_closes_every_notice_on_screen() {
    let mut bench = Bench::new("hide", false);
    bench.owner_looks_at_rex(NOW);
    raise_waiting(&mut bench, "agent-1");
    bench.raise("tests", Level::Finished, NOW);
    bench.calls.screen.taken();

    let answer = bench.desk.answer(NoticeRequest::Hide, NOW + 1);

    assert_eq!(answer, Ok(json!({ "hidden": true })));
    assert_eq!(bench.calls.screen.taken().len(), 2);
}

#[test]
fn jump_moves_the_owner_to_the_notices_pane_and_marks_it_read() {
    let mut bench = Bench::new("jump", false);
    raise_waiting(&mut bench, "agent-1");

    let answer = bench.desk.answer(NoticeRequest::Jump { id: None }, NOW + 1);

    assert_eq!(answer, Ok(json!({ "jumped": true })));
    assert_eq!(
        bench.calls.screen.taken(),
        vec!["jump block:00000000-0000-4000-8000-0000000000b2"]
    );
    let rows = read_inbox(&bench.inbox).unwrap_or_else(|failure| panic!("{failure}"));
    assert_eq!(rows[0].read_at, Some(NOW + 1));
}

#[test]
fn dismiss_marks_the_row_read_even_when_the_board_forgot_it() {
    let mut bench = Bench::new("dismiss", false);
    raise_waiting(&mut bench, "agent-1");
    let mut forgotten = read_inbox(&bench.inbox).unwrap_or_default()[0].clone();
    forgotten.id = "1780000000-1".to_string();
    save_inbox_row(&bench.inbox, &forgotten).unwrap_or_else(|failure| panic!("{failure}"));

    let dismissed = bench.desk.answer(
        NoticeRequest::Dismiss {
            id: forgotten.id.clone(),
        },
        NOW + 1,
    );
    let unknown = bench.desk.answer(
        NoticeRequest::Read {
            id: "never".to_string(),
        },
        NOW + 1,
    );

    assert_eq!(dismissed, Ok(json!({ "dismissed": true })));
    assert_eq!(unknown, Ok(json!({ "read": false })));
    let rows = read_inbox(&bench.inbox).unwrap_or_default();
    assert_eq!(rows[1].read_at, Some(NOW + 1));
}

#[test]
fn a_muted_source_stays_off_the_screen_until_unmuted() {
    let mut bench = Bench::new("mute", false);
    bench.owner_looks_at_rex(NOW);
    let mute = NoticeRequest::Mute {
        source: "tests".to_string(),
        until: Some(NOW + 600),
    };

    let muted = bench.desk.answer(mute, NOW);
    bench.raise("first", Level::Finished, NOW);
    let while_muted = bench.calls.screen.taken();
    let unmuted = bench.desk.answer(
        NoticeRequest::Unmute {
            source: "tests".to_string(),
        },
        NOW,
    );
    bench.raise("second", Level::Finished, NOW);

    assert_eq!(muted, Ok(json!({ "muted": "tests", "until": NOW + 600 })));
    assert_eq!(unmuted, Ok(json!({ "unmuted": "tests" })));
    assert_eq!(while_muted, Vec::<String>::new());
    assert_eq!(bench.calls.screen.taken().len(), 1);
}
