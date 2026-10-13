use super::*;
use herdr_workspace_jump_domain::RowStyle;

#[test]
fn stripes_shade_every_other_row_to_the_edge_and_draw_no_leaders() {
    let workspaces = three_workspaces();
    let layout = layout().rows(RowStyle::Stripes);

    let drawn = render(&browsing(&workspaces, 0), &layout, 32, 0);

    assert_eq!(
        drawn.lines().nth(3),
        Some(
            "\x1b[48;5;235m\x1b[1;34m   h\x1b[22;39m  homelab    \x1b[2m/opt/homelab\x1b[22;39m   \x1b[0m"
        )
    );
    assert_eq!(
        drawn.lines().nth(4),
        Some("\x1b[1;34m   f\x1b[22;39m  dotfiles   \x1b[2m/opt/dotfiles\x1b[22;39m")
    );
    assert_eq!(
        visible_rows(&drawn)[1..4],
        [
            "i  Ivy        /opt/Ivy",
            "h  homelab    /opt/homelab",
            "f  dotfiles   /opt/dotfiles"
        ]
    );
}

#[test]
fn the_selected_row_keeps_its_bar_on_a_striped_row() {
    let workspaces = three_workspaces();
    let layout = layout().rows(RowStyle::Stripes);

    let drawn = render(&browsing(&workspaces, 1), &layout, 32, 0);

    assert!(
        drawn
            .lines()
            .nth(3)
            .is_some_and(|row| row.starts_with("\x1b[48;5;236m")),
        "{drawn:?}"
    );
}
