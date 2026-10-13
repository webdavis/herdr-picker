use super::*;

#[test]
fn a_refused_key_says_so_in_red_on_the_row_above_the_legend_which_stays_put() {
    let workspaces = three_workspaces();

    let drawn = render(
        &refused(&workspaces[..2], &workspaces[2]),
        LEGEND,
        columns(),
        0,
        0,
    );

    let mut rows = drawn.lines().rev();
    assert_eq!(rows.next(), Some(LEGEND_ROW));
    assert_eq!(rows.next(), Some("\x1b[31m already in dotfiles\x1b[0m"));
    assert_eq!(drawn.lines().count(), 6);
}

#[test]
fn a_refusal_is_cut_to_a_narrow_popup_like_the_legend() {
    let workspaces = three_workspaces();

    let drawn = render(
        &refused(&workspaces[..2], &workspaces[2]),
        LEGEND,
        columns(),
        12,
        0,
    );

    assert_eq!(
        drawn.lines().rev().nth(1),
        Some("\x1b[31m already in\u{2026}\x1b[0m")
    );
}
