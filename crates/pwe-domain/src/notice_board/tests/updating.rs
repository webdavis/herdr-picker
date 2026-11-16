use super::*;

fn relabelled(effects: &[NoticeEffect]) -> Vec<(String, String, u64)> {
    effects
        .iter()
        .filter_map(|effect| match effect {
            NoticeEffect::Relabel {
                float,
                line,
                seconds,
            } => Some((float.clone(), line.clone(), *seconds)),
            _ => None,
        })
        .collect()
}

fn running_tests() -> Raised {
    Raised {
        text: "Tests running".to_string(),
        ..raised("tests", Level::Finished)
    }
}

fn failed_tests() -> Raised {
    Raised {
        text: "Tests failed: 3".to_string(),
        ..raised("tests", Level::Finished)
    }
}

#[test]
fn a_running_thing_changes_its_own_notice_in_place() {
    let mut board = board_in_front();
    let float = &shown(&board.apply(NoticeInput::Raise(running_tests()), 100))[0];

    let effects = board.apply(NoticeInput::Raise(failed_tests()), 104);

    assert!(shown(&effects).is_empty());
    assert!(closed(&effects).is_empty());
    assert_eq!(
        relabelled(&effects),
        vec![(float.float.clone(), "Tests failed: 3".to_string(), 6)]
    );
}

#[test]
fn an_update_keeps_the_notice_and_its_raised_time() {
    let mut board = board_in_front();
    let first = &saved_rows(&board.apply(NoticeInput::Raise(running_tests()), 100))[0];

    let updated = &saved_rows(&board.apply(NoticeInput::Raise(failed_tests()), 104))[0];

    assert_eq!(
        (&updated.id, updated.raised_at, updated.updated_at),
        (&first.id, 100, 104)
    );
    assert_eq!(updated.text, "Tests failed: 3");
}

#[test]
fn an_updated_notice_that_had_left_the_screen_is_shown_again() {
    let mut board = board_in_front();
    board.apply(NoticeInput::Raise(running_tests()), 100);
    board.apply(NoticeInput::TimePassed, 106);

    let effects = board.apply(NoticeInput::Raise(failed_tests()), 200);

    assert_eq!(shown(&effects)[0].line, "Tests failed: 3");
}

#[test]
fn an_update_that_changes_the_level_reopens_the_notice_in_its_new_level() {
    let mut board = board_in_front();
    let float = &shown(&board.apply(NoticeInput::Raise(running_tests()), 100))[0];
    let blocked = Raised {
        level: Level::NeedsYou,
        ..failed_tests()
    };

    let effects = board.apply(NoticeInput::Raise(blocked), 104);

    assert_eq!(closed(&effects), vec![float.float.clone()]);
    assert_eq!(shown(&effects)[0].level, Level::NeedsYou);
}

#[test]
fn a_waiting_notice_removes_itself_when_the_thing_moves_on() {
    let mut board = board_in_front();
    let float = &shown(&board.apply(raise("agent", Level::NeedsYou), 100))[0];

    let effects = board.apply(
        NoticeInput::Resolve {
            key: "agent".to_string(),
        },
        103,
    );

    assert_eq!(closed(&effects), vec![float.float.clone()]);
    assert_eq!(saved_rows(&effects)[0].resolved_at, Some(103));
}

#[test]
fn a_key_raised_again_after_it_resolved_is_a_new_notice() {
    let mut board = board_in_front();
    let first = &saved_rows(&board.apply(raise("agent", Level::NeedsYou), 100))[0];
    board.apply(
        NoticeInput::Resolve {
            key: "agent".to_string(),
        },
        103,
    );

    let again = &saved_rows(&board.apply(raise("agent", Level::NeedsYou), 110))[0];

    assert_ne!(again.id, first.id);
}
