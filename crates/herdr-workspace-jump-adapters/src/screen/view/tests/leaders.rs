use super::*;
use herdr_workspace_jump_domain::RowLook;

const LEADERS_ONLY: RowLook = RowLook {
    leaders: true,
    stripes: false,
};

#[test]
fn leaders_switched_on_run_from_each_name_to_the_directory_column_with_a_space_either_side() {
    let workspaces = three_workspaces();
    let layout = layout().look(LEADERS_ONLY);

    let drawn = render(&browsing(&workspaces, 0), &layout, 0, 0);

    assert_eq!(
        drawn.lines().nth(3),
        Some(
            "\x1b[1;38;2;137;180;250m   h\x1b[22;39m  homelab\x1b[38;2;69;71;90m ·· \x1b[22;39m\x1b[38;2;127;132;156m/opt/homelab\x1b[22;39m"
        )
    );
    assert_eq!(
        visible_rows(&drawn)[1..4],
        [
            "i  Ivy ······ /opt/Ivy",
            "h  homelab ·· /opt/homelab",
            "f  dotfiles · /opt/dotfiles"
        ]
    );
}

#[test]
fn the_directory_column_is_set_by_the_widest_name_offered_not_by_a_hidden_one() {
    let workspaces = three_workspaces();
    let layout = layout().look(LEADERS_ONLY).hiding(Some("dotfiles"));

    let drawn = render(&browsing(&workspaces[..2], 0), &layout, 0, 0);

    assert_eq!(
        visible_rows(&drawn)[1..3],
        ["i  Ivy ····· /opt/Ivy", "h  homelab · /opt/homelab"]
    );
}
