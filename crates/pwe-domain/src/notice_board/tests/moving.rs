use super::*;

fn agent_in(key: &str, block: &str) -> NoticeInput {
    NoticeInput::Raise(Raised {
        kind: "agent".to_string(),
        pane: Some(pane(block)),
        ..raised(key, Level::NeedsYou)
    })
}

fn move_to(pane: Pane) -> NoticeInput {
    NoticeInput::OwnerMoved(RexFocus::Front(pane))
}

fn lines(effects: &[NoticeEffect]) -> Vec<String> {
    shown(effects).into_iter().map(|shown| shown.line).collect()
}

fn board_with_two_waiting_agents() -> NoticeBoard {
    let mut board = board_in_front();
    board.apply(agent_in("one", "left"), 100);
    board.apply(agent_in("two", "right"), 101);
    board.apply(NoticeInput::TimePassed, 200);
    board
}

#[test]
fn nothing_is_mentioned_again_on_a_timer() {
    let mut board = board_with_two_waiting_agents();

    let effects = board.apply(NoticeInput::TimePassed, 5000);

    assert!(effects.is_empty());
}

#[test]
fn switching_tab_mentions_in_one_line_how_many_agents_wait() {
    let mut board = board_with_two_waiting_agents();

    let effects = board.apply(move_to(pane_in("work", "web", "shell")), 300);

    assert_eq!(lines(&effects), vec!["2 agents waiting".to_string()]);
}

#[test]
fn switching_session_mentions_what_waits() {
    let mut board = board_in_front();
    board.apply(agent_in("one", "left"), 100);

    let effects = board.apply(move_to(pane_in("home", "api", "shell")), 300);

    assert_eq!(lines(&effects), vec!["1 agent waiting".to_string()]);
}

#[test]
fn waiting_notices_of_different_kinds_are_counted_together() {
    let mut board = board_with_two_waiting_agents();
    board.apply(raise("deploy", Level::NeedsYou), 102);

    let effects = board.apply(move_to(pane_in("work", "web", "shell")), 300);

    assert_eq!(lines(&effects), vec!["3 waiting".to_string()]);
}

#[test]
fn moving_to_another_pane_of_the_same_tab_mentions_nothing() {
    let mut board = board_with_two_waiting_agents();

    assert!(shown(&board.apply(move_to(pane("other")), 300)).is_empty());
}

#[test]
fn coming_back_to_rex_mentions_nothing() {
    let mut board = board_with_two_waiting_agents();
    board.apply(NoticeInput::OwnerMoved(RexFocus::Background), 300);

    assert!(shown(&board.apply(move_to(pane("view")), 400)).is_empty());
}

#[test]
fn a_finished_notice_is_never_mentioned_again() {
    let mut board = board_in_front();
    board.apply(raise("build", Level::Finished), 100);

    let effects = board.apply(move_to(pane_in("work", "web", "shell")), 300);

    assert!(shown(&effects).is_empty());
}

#[test]
fn the_agent_in_the_pane_he_lands_on_is_left_out_of_the_count() {
    let mut board = board_with_two_waiting_agents();

    let effects = board.apply(move_to(pane_in("work", "api", "left")), 300);

    assert!(shown(&effects).is_empty());
    let effects = board.apply(move_to(pane_in("work", "web", "shell")), 400);
    assert_eq!(lines(&effects), vec!["2 agents waiting".to_string()]);
}

#[test]
fn one_request_lists_every_notice_that_still_waits_newest_first() {
    let mut board = board_with_two_waiting_agents();
    board.apply(raise("build", Level::Finished), 150);

    let effects = board.apply(NoticeInput::ListWaiting, 300);

    let [NoticeEffect::Waiting(waiting)] = effects.as_slice() else {
        panic!("expected one waiting list, got {effects:?}");
    };
    let keys: Vec<&str> = waiting.iter().map(|notice| notice.key.as_str()).collect();
    assert_eq!(keys, vec!["two", "one"]);
}

#[test]
fn the_session_is_asked_for_only_while_a_notice_waits() {
    let mut board = board_in_front();

    let raised = board.apply(agent_in("one", "left"), 100);
    let resolved = board.apply(
        NoticeInput::Resolve {
            key: "one".to_string(),
        },
        110,
    );

    assert!(raised.contains(&NoticeEffect::AskForSession { every_seconds: 2 }));
    assert!(resolved.contains(&NoticeEffect::StopAskingForSession));
    assert!(
        !board
            .apply(raise("build", Level::Finished), 120)
            .iter()
            .any(|effect| matches!(effect, NoticeEffect::AskForSession { .. }))
    );
}

#[test]
fn a_finished_notice_counts_as_read_once_its_pane_has_had_focus_for_a_second() {
    let mut board = board_in_front();
    board.apply(
        NoticeInput::Raise(Raised {
            pane: Some(pane("left")),
            ..raised("build", Level::Finished)
        }),
        100,
    );
    board.apply(move_to(pane("left")), 200);

    assert!(saved_rows(&board.apply(NoticeInput::TimePassed, 200)).is_empty());
    let rows = saved_rows(&board.apply(NoticeInput::TimePassed, 201));
    assert_eq!(rows[0].read_at, Some(201));
}

#[test]
fn a_needs_you_notice_stays_unread_while_he_looks_at_its_pane() {
    let mut board = board_in_front();
    board.apply(agent_in("one", "left"), 100);
    board.apply(move_to(pane("left")), 200);

    assert!(saved_rows(&board.apply(NoticeInput::TimePassed, 300)).is_empty());
}
