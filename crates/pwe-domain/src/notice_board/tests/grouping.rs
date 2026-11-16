use super::*;

fn build(key: &str) -> NoticeInput {
    NoticeInput::Raise(Raised {
        kind: "build".to_string(),
        ..raised(key, Level::Finished)
    })
}

fn agent(key: &str) -> NoticeInput {
    NoticeInput::Raise(Raised {
        kind: "agent".to_string(),
        text: format!("{key} asks"),
        ..raised(key, Level::NeedsYou)
    })
}

fn relabelled(effects: &[NoticeEffect]) -> Vec<(String, String)> {
    effects
        .iter()
        .filter_map(|effect| match effect {
            NoticeEffect::Relabel { float, line, .. } => Some((float.clone(), line.clone())),
            _ => None,
        })
        .collect()
}

#[test]
fn several_notices_of_one_kind_become_one_line() {
    let mut board = board_in_front();
    let float = shown(&board.apply(agent("one"), 100))[0].float.clone();

    let second = board.apply(agent("two"), 101);
    let third = board.apply(agent("three"), 102);

    assert!(shown(&second).is_empty() && shown(&third).is_empty());
    assert_eq!(
        relabelled(&second),
        vec![(float.clone(), "2 agents waiting".to_string())]
    );
    assert_eq!(
        relabelled(&third),
        vec![(float, "3 agents waiting".to_string())]
    );
}

#[test]
fn finished_notices_of_one_kind_are_counted_as_finished() {
    let mut board = board_in_front();
    board.apply(build("one"), 100);

    let effects = board.apply(build("two"), 101);

    assert_eq!(relabelled(&effects)[0].1, "2 builds finished");
}

#[test]
fn notices_of_different_kinds_keep_their_own_lines() {
    let mut board = board_in_front();
    board.apply(agent("one"), 100);

    let effects = board.apply(raise("build", Level::NeedsYou), 101);

    assert_eq!(shown(&effects).len(), 1);
    assert!(relabelled(&effects).is_empty());
}

#[test]
fn a_group_keeps_its_slot_so_the_screen_never_holds_more_than_three() {
    let mut board = board_in_front();
    for key in ["one", "two", "three", "four", "five"] {
        board.apply(agent(key), 100);
    }

    let effects = board.apply(raise("build", Level::NeedsYou), 101);

    assert!(closed(&effects).is_empty());
    assert_eq!(shown(&effects)[0].slot, 1);
}

#[test]
fn a_resolved_notice_leaves_its_group_and_the_line_counts_the_rest() {
    let mut board = board_in_front();
    let float = shown(&board.apply(agent("one"), 100))[0].float.clone();
    board.apply(agent("two"), 101);

    let effects = board.apply(
        NoticeInput::Resolve {
            key: "two".to_string(),
        },
        102,
    );

    assert!(closed(&effects).is_empty());
    assert_eq!(
        relabelled(&effects),
        vec![(float, "needs you · tests: one asks · 2s ago".to_string())]
    );
}
