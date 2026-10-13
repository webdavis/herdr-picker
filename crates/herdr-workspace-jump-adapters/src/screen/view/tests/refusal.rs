use super::*;

#[test]
fn a_refused_key_says_so_in_red_on_the_row_above_the_legend_which_stays_put() {
    let workspaces = three_workspaces();

    let drawn = render(&refused(&workspaces[..2], &workspaces[2]), &layout(), 0, 0);

    let mut rows = drawn.lines().rev();
    assert_eq!(rows.next(), Some(LEGEND_ROW));
    assert_eq!(rows.next(), Some("\x1b[31m already in dotfiles\x1b[0m"));
    assert_eq!(drawn.lines().count(), 6);
}

#[test]
fn a_notice_sits_faint_on_the_row_above_the_legend_until_a_refusal_takes_its_place() {
    let workspaces = three_workspaces();
    const NOTICE: &str = "herdr did not answer";

    let quiet = render(&browsing(&workspaces, 0), &layout().noting(NOTICE), 0, 0);
    assert_eq!(
        quiet.lines().rev().nth(1),
        Some("\x1b[2m herdr did not answer\x1b[0m")
    );

    let refused = render(
        &refused(&workspaces[..2], &workspaces[2]),
        &layout().noting(NOTICE),
        0,
        0,
    );
    assert_eq!(
        refused.lines().rev().nth(1),
        Some("\x1b[31m already in dotfiles\x1b[0m")
    );
}

#[test]
fn a_refusal_is_cut_to_a_narrow_popup_like_the_legend() {
    let workspaces = three_workspaces();

    let drawn = render(&refused(&workspaces[..2], &workspaces[2]), &layout(), 12, 0);

    assert_eq!(
        drawn.lines().rev().nth(1),
        Some("\x1b[31m already in\u{2026}\x1b[0m")
    );
}
