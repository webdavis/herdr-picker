use super::*;

fn failed_tests_in_tab_api() -> Raised {
    Raised {
        source: "cargo".to_string(),
        text: "Tests failed: 3".to_string(),
        pane: Some(pane("left")),
        tab_label: Some("api".to_string()),
        ..raised("tests", Level::NeedsYou)
    }
}

fn jump(id: Option<String>) -> NoticeInput {
    NoticeInput::Jump { id }
}

fn jumped(effects: &[NoticeEffect]) -> Vec<Pane> {
    effects
        .iter()
        .filter_map(|effect| match effect {
            NoticeEffect::JumpTo(pane) => Some(pane.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn a_line_names_its_level_its_source_and_its_tab() {
    let mut board = board_in_front();

    let effects = board.apply(NoticeInput::Raise(failed_tests_in_tab_api()), 100);

    assert_eq!(
        shown(&effects)[0].line,
        "needs you · cargo: Tests failed: 3, in tab api · now"
    );
}

#[test]
fn a_line_without_a_tab_names_only_its_source() {
    let mut board = board_in_front();

    let effects = board.apply(raise("build", Level::Finished), 100);

    assert_eq!(
        shown(&effects)[0].line,
        "finished · tests: build done · now"
    );
}

#[test]
fn a_line_says_how_long_ago_its_notice_changed() {
    let mut board = board_in_front();
    board.apply(NoticeInput::Raise(failed_tests_in_tab_api()), 100);
    board.apply(
        NoticeInput::Raise(Raised {
            key: "lint".to_string(),
            ..failed_tests_in_tab_api()
        }),
        101,
    );

    let effects = board.apply(
        NoticeInput::Resolve {
            key: "lint".to_string(),
        },
        105,
    );

    let [_, NoticeEffect::Relabel { line, .. }] = effects.as_slice() else {
        panic!("expected a relabel, got {effects:?}");
    };
    assert!(line.ends_with(" · 5s ago"), "{line}");
}

#[test]
fn a_label_is_cut_to_128_characters_without_control_characters() {
    let mut board = board_in_front();
    let noisy = Raised {
        text: format!("\u{1b}[31mred\u{7}\n{}", "x".repeat(300)),
        ..raised("noisy", Level::Finished)
    };

    let line = shown(&board.apply(NoticeInput::Raise(noisy), 100))[0]
        .line
        .clone();

    assert_eq!(line.chars().count(), 128);
    assert!(!line.chars().any(char::is_control));
    assert!(line.starts_with("finished · tests: [31mredx"));
    assert!(line.ends_with('…'));
}

#[test]
fn jumping_to_a_notice_goes_to_its_pane_and_marks_it_read() {
    let mut board = board_in_front();
    let id = saved_rows(&board.apply(NoticeInput::Raise(failed_tests_in_tab_api()), 100))[0]
        .id
        .clone();

    let effects = board.apply(jump(Some(id)), 103);

    assert_eq!(jumped(&effects), vec![pane("left")]);
    assert_eq!(saved_rows(&effects)[0].read_at, Some(103));
}

#[test]
fn jumping_without_an_id_goes_to_the_newest_waiting_notice() {
    let mut board = board_in_front();
    board.apply(NoticeInput::Raise(failed_tests_in_tab_api()), 100);
    board.apply(
        NoticeInput::Raise(Raised {
            key: "agent".to_string(),
            pane: Some(pane("right")),
            ..failed_tests_in_tab_api()
        }),
        101,
    );
    board.apply(
        NoticeInput::Raise(Raised {
            pane: Some(pane("bottom")),
            ..raised("build", Level::Finished)
        }),
        102,
    );

    assert_eq!(jumped(&board.apply(jump(None), 103)), vec![pane("right")]);
}

#[test]
fn a_notice_without_a_pane_has_nowhere_to_jump() {
    let mut board = board_in_front();
    let id = saved_rows(&board.apply(raise("build", Level::Finished), 100))[0]
        .id
        .clone();

    assert!(board.apply(jump(Some(id)), 101).is_empty());
}

#[test]
fn when_rex_is_in_the_background_a_notice_goes_to_macos() {
    let mut board = board_in_front();
    board.apply(NoticeInput::OwnerMoved(RexFocus::Background), 100);

    let effects = board.apply(NoticeInput::Raise(failed_tests_in_tab_api()), 101);

    assert!(shown(&effects).is_empty());
    assert!(effects.contains(&NoticeEffect::SendToMacos {
        title: "needs you".to_string(),
        text: "cargo: Tests failed: 3, in tab api".to_string(),
    }));
}

#[test]
fn when_rex_is_closed_a_notice_goes_to_macos_and_a_quiet_one_does_not() {
    let mut board = NoticeBoard::new(NoticeSettings::default());
    board.apply(NoticeInput::OwnerMoved(RexFocus::Closed), 100);

    let finished = board.apply(raise("build", Level::Finished), 101);
    let quiet = board.apply(raise("sync", Level::Quiet), 102);

    let to_macos = |effects: &[NoticeEffect]| {
        effects
            .iter()
            .filter(|effect| matches!(effect, NoticeEffect::SendToMacos { .. }))
            .count()
    };
    assert_eq!((to_macos(&finished), to_macos(&quiet)), (1, 0));
}
