use super::*;

fn from(source: &str, key: &str) -> NoticeInput {
    NoticeInput::Raise(Raised {
        source: source.to_string(),
        ..raised(key, Level::NeedsYou)
    })
}

fn mute(source: &str, until: Option<u64>) -> NoticeInput {
    NoticeInput::Mute {
        source: source.to_string(),
        until,
    }
}

fn first_id(effects: &[NoticeEffect]) -> String {
    saved_rows(effects)[0].id.clone()
}

#[test]
fn one_hide_closes_every_notice_on_screen_at_once_and_they_still_wait() {
    let mut board = board_in_front();
    board.apply(from("claude-code", "one"), 100);
    board.apply(from("codex", "two"), 101);

    let effects = board.apply(NoticeInput::Hide, 102);

    assert_eq!(closed(&effects).len(), 2);
    let listed = board.apply(NoticeInput::ListWaiting, 103);
    assert!(matches!(listed.as_slice(), [NoticeEffect::Waiting(waiting)] if waiting.len() == 2));
}

#[test]
fn a_muted_source_still_gets_its_inbox_row_and_shows_nothing() {
    let mut board = board_in_front();
    board.apply(mute("codex", None), 100);

    let effects = board.apply(from("codex", "one"), 101);

    assert!(shown(&effects).is_empty());
    assert_eq!(saved_rows(&effects).len(), 1);
}

#[test]
fn a_mute_for_a_set_time_ends_by_itself() {
    let mut board = board_in_front();
    board.apply(mute("codex", Some(200)), 100);

    assert!(shown(&board.apply(from("codex", "one"), 199)).is_empty());
    assert_eq!(shown(&board.apply(from("codex", "two"), 200)).len(), 1);
}

#[test]
fn a_star_mutes_every_source() {
    let mut board = board_in_front();
    board.apply(mute("*", None), 100);

    assert!(shown(&board.apply(from("claude-code", "one"), 101)).is_empty());
    assert!(shown(&board.apply(from("codex", "two"), 102)).is_empty());
}

#[test]
fn unmuting_a_source_shows_its_notices_again() {
    let mut board = board_in_front();
    board.apply(mute("codex", None), 100);
    board.apply(
        NoticeInput::Unmute {
            source: "codex".to_string(),
        },
        101,
    );

    assert_eq!(shown(&board.apply(from("codex", "one"), 102)).len(), 1);
}

#[test]
fn muting_a_source_closes_its_notices_on_screen() {
    let mut board = board_in_front();
    let float = shown(&board.apply(from("codex", "one"), 100))[0]
        .float
        .clone();
    board.apply(from("claude-code", "two"), 101);

    let effects = board.apply(mute("codex", None), 102);

    assert_eq!(closed(&effects), vec![float]);
}

#[test]
fn a_muted_source_is_left_out_when_what_waits_is_mentioned() {
    let mut board = board_in_front();
    board.apply(from("codex", "one"), 100);
    board.apply(mute("codex", None), 101);

    let effects = board.apply(
        NoticeInput::OwnerMoved(RexFocus::Front(pane_in("work", "web", "shell"))),
        300,
    );

    assert!(shown(&effects).is_empty());
}

#[test]
fn dismissing_a_notice_marks_its_inbox_row_read() {
    let mut board = board_in_front();
    let raised = board.apply(from("codex", "one"), 100);
    let float = shown(&raised)[0].float.clone();

    let effects = board.apply(
        NoticeInput::Dismiss {
            id: first_id(&raised),
        },
        104,
    );

    assert_eq!(closed(&effects), vec![float]);
    assert_eq!(saved_rows(&effects)[0].read_at, Some(104));
}

#[test]
fn reading_the_row_removes_the_notice() {
    let mut board = board_in_front();
    let raised = board.apply(from("codex", "one"), 100);
    let float = shown(&raised)[0].float.clone();

    let effects = board.apply(
        NoticeInput::Read {
            id: first_id(&raised),
        },
        104,
    );

    assert_eq!(closed(&effects), vec![float]);
    let listed = board.apply(NoticeInput::ListWaiting, 105);
    assert!(matches!(listed.as_slice(), [NoticeEffect::Waiting(waiting)] if waiting.is_empty()));
}
