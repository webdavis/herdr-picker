use super::*;

#[test]
fn every_notice_is_saved_as_an_unread_inbox_row() {
    let mut board = board_in_front();

    let rows = saved_rows(&board.apply(raise("build", Level::Finished), 100));

    assert_eq!(rows.len(), 1);
    assert_eq!(
        (rows[0].key.as_str(), rows[0].raised_at, rows[0].is_unread()),
        ("build", 100, true)
    );
}

#[test]
fn a_finished_notice_closes_after_its_seconds() {
    let mut board = board_in_front();
    let float = &shown(&board.apply(raise("build", Level::Finished), 100))[0];

    assert_eq!(float.seconds, 6);
    assert!(closed(&board.apply(NoticeInput::TimePassed, 105)).is_empty());
    assert_eq!(
        closed(&board.apply(NoticeInput::TimePassed, 106)),
        vec![float.float.clone()]
    );
}

#[test]
fn a_needs_you_notice_also_leaves_by_itself_after_its_longer_seconds() {
    let mut board = board_in_front();
    let float = &shown(&board.apply(raise("agent", Level::NeedsYou), 100))[0];

    assert_eq!(float.seconds, 10);
    assert!(closed(&board.apply(NoticeInput::TimePassed, 109)).is_empty());
    assert_eq!(
        closed(&board.apply(NoticeInput::TimePassed, 110)),
        vec![float.float.clone()]
    );
}

#[test]
fn a_quiet_notice_goes_to_the_inbox_only() {
    let mut board = board_in_front();

    let effects = board.apply(raise("sync", Level::Quiet), 100);

    assert_eq!(effects.len(), 1);
    assert_eq!(saved_rows(&effects).len(), 1);
}

#[test]
fn a_notice_about_the_pane_in_view_stays_silent() {
    let mut board = board_in_front();
    let about_view = Raised {
        pane: Some(pane("view")),
        ..raised("agent", Level::NeedsYou)
    };

    let effects = board.apply(NoticeInput::Raise(about_view), 100);

    assert_eq!(effects.len(), 1);
    assert_eq!(saved_rows(&effects).len(), 1);
}

#[test]
fn a_notice_about_another_pane_in_the_same_tab_is_shown() {
    let mut board = board_in_front();
    let about_neighbour = Raised {
        pane: Some(pane("neighbour")),
        ..raised("agent", Level::NeedsYou)
    };

    assert_eq!(
        shown(&board.apply(NoticeInput::Raise(about_neighbour), 100)).len(),
        1
    );
}

#[test]
fn each_notice_on_screen_takes_the_next_free_slot_in_the_corner() {
    let mut board = board_in_front();
    let first = &shown(&board.apply(raise("first", Level::Finished), 100))[0];
    let second = &shown(&board.apply(raise("second", Level::Finished), 103))[0];
    board.apply(NoticeInput::TimePassed, 106);
    let third = &shown(&board.apply(raise("third", Level::Finished), 107))[0];

    assert_eq!((first.slot, second.slot, third.slot), (0, 1, 0));
    assert_eq!(third.line, "third done");
}

#[test]
fn needs_you_is_shown_before_finished() {
    let mut board = board_in_front();
    let oldest = &shown(&board.apply(raise("one", Level::Finished), 100))[0];
    board.apply(raise("two", Level::NeedsYou), 101);
    board.apply(raise("three", Level::Finished), 102);

    let effects = board.apply(raise("blocked", Level::NeedsYou), 103);

    assert_eq!(closed(&effects), vec![oldest.float.clone()]);
    assert_eq!(shown(&effects)[0].slot, oldest.slot);
    assert_eq!(shown(&effects)[0].level, Level::NeedsYou);
}

#[test]
fn a_finished_notice_never_pushes_a_needs_you_notice_off_the_screen() {
    let mut board = board_in_front();
    for key in ["one", "two", "three"] {
        board.apply(raise(key, Level::NeedsYou), 100);
    }

    let effects = board.apply(raise("build", Level::Finished), 101);

    assert!(closed(&effects).is_empty());
    assert!(shown(&effects).is_empty());
    assert_eq!(saved_rows(&effects).len(), 1);
}

#[test]
fn at_most_three_notices_are_on_screen_and_a_fourth_closes_the_oldest() {
    let mut board = board_in_front();
    let oldest = &shown(&board.apply(raise("one", Level::Finished), 100))[0];
    board.apply(raise("two", Level::Finished), 101);
    board.apply(raise("three", Level::Finished), 102);

    let effects = board.apply(raise("four", Level::Finished), 103);

    assert_eq!(closed(&effects), vec![oldest.float.clone()]);
    assert_eq!(shown(&effects)[0].slot, oldest.slot);
}
