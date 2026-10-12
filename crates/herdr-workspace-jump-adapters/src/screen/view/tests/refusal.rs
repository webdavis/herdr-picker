use super::*;

#[test]
fn a_refused_key_says_so_in_red_on_the_bottom_row_in_place_of_the_legend() {
    let workspaces = three_workspaces();

    let drawn = render(&refused(&workspaces[..2], &workspaces[2]), LEGEND, 0, 0);

    assert_eq!(
        drawn.lines().last(),
        Some("\x1b[31m already in dotfiles\x1b[0m")
    );
    assert!(!drawn.contains("the keys"), "{drawn:?}");
    assert_eq!(drawn.lines().count(), 6);
}

#[test]
fn a_refusal_is_cut_to_a_narrow_popup_like_the_legend() {
    let workspaces = three_workspaces();

    let drawn = render(&refused(&workspaces[..2], &workspaces[2]), LEGEND, 12, 0);

    assert_eq!(
        drawn.lines().last(),
        Some("\x1b[31m already in\u{2026}\x1b[0m")
    );
}
