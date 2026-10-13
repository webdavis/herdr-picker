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
        Some(format!(
            "{ON_BASE}{BOLD_BLUE}   h{PLAIN}  homelab{DARK_GREY} ·· {PLAIN}{GREY}/opt/homelab{PLAIN}{END}"
        ))
        .as_deref()
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
