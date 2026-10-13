use super::*;
use herdr_workspace_jump_domain::RowLook;

const STRIPES_ONLY: RowLook = RowLook {
    leaders: false,
    stripes: true,
};

#[test]
fn stripes_shade_every_other_row_to_the_edge_and_leaders_off_leave_a_plain_gap() {
    let workspaces = three_workspaces();
    let layout = layout().look(STRIPES_ONLY);

    let drawn = render(&browsing(&workspaces, 0), &layout, 32, 0);

    assert_eq!(
        drawn.lines().nth(3),
        Some(
            "\x1b[48;2;33;33;52m\x1b[1;38;2;137;180;250m   h\x1b[22;39m  homelab    \x1b[38;2;127;132;156m/opt/homelab\x1b[22;39m   \x1b[0m"
        )
    );
    assert_eq!(
        drawn.lines().nth(4),
        Some(
            "\x1b[1;38;2;137;180;250m   f\x1b[22;39m  dotfiles   \x1b[38;2;127;132;156m/opt/dotfiles\x1b[22;39m"
        )
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
fn both_switches_on_draw_leaders_across_the_shaded_rows_too() {
    let workspaces = three_workspaces();
    let layout = layout().look(RowLook {
        leaders: true,
        stripes: true,
    });

    let drawn = render(&browsing(&workspaces, 0), &layout, 32, 0);

    assert_eq!(
        drawn.lines().nth(3),
        Some(
            "\x1b[48;2;33;33;52m\x1b[1;38;2;137;180;250m   h\x1b[22;39m  homelab\x1b[38;2;69;71;90m ·· \x1b[22;39m\x1b[38;2;127;132;156m/opt/homelab\x1b[22;39m   \x1b[0m"
        )
    );
}

#[test]
fn both_switches_off_leave_plain_rows_with_a_plain_gap() {
    let workspaces = three_workspaces();
    let layout = layout().look(RowLook {
        leaders: false,
        stripes: false,
    });

    let drawn = render(&browsing(&workspaces, 0), &layout, 32, 0);

    assert_eq!(
        drawn.lines().nth(3),
        Some(
            "\x1b[1;38;2;137;180;250m   h\x1b[22;39m  homelab    \x1b[38;2;127;132;156m/opt/homelab\x1b[22;39m"
        )
    );
}

#[test]
fn the_selected_row_keeps_its_bar_on_a_striped_row() {
    let workspaces = three_workspaces();
    let layout = layout().look(STRIPES_ONLY);

    let drawn = render(&browsing(&workspaces, 1), &layout, 32, 0);

    assert!(
        drawn
            .lines()
            .nth(3)
            .is_some_and(|row| row.starts_with("\x1b[48;2;54;54;70m")),
        "{drawn:?}"
    );
}
