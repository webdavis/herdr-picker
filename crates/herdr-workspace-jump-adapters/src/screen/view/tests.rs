use super::*;
use crate::screen::fixtures::{browsing, three_workspaces};

#[test]
fn a_frame_is_every_row_with_a_grey_bar_on_the_selected_one_a_blank_and_the_key_legend() {
    let workspaces = three_workspaces();

    assert_eq!(
        render(&browsing(&workspaces, 0, None), 20),
        "\x1b[48;5;236m \x1b[1;34m  i\x1b[0m\x1b[48;5;236m  Ivy           \x1b[0m\n \
         \x1b[1;34m  h\x1b[0m  homelab\n \
         \x1b[1;34m  f\x1b[0m  dotfiles\n\
         \n\
         \x1b[2m ctrl+n/p move  enter jump  esc close\x1b[0m"
    );
}

#[test]
fn the_current_workspace_is_faint_and_says_so_at_the_right_edge_and_faint_on_the_bar() {
    let workspaces = three_workspaces();

    assert_eq!(
        render(&browsing(&workspaces, 0, Some(1)), 24)
            .lines()
            .nth(1),
        Some("\x1b[2m   h  homelab   current \x1b[0m")
    );
    assert_eq!(
        render(&browsing(&workspaces, 1, Some(1)), 24)
            .lines()
            .nth(1),
        Some("\x1b[48;5;236m\x1b[2m   h  homelab   current \x1b[0m")
    );
}

#[test]
fn a_width_too_narrow_for_the_row_pads_nothing_and_cuts_nothing() {
    let workspaces = three_workspaces();

    assert_eq!(
        render(&browsing(&workspaces, 0, None), 0).lines().next(),
        Some("\x1b[48;5;236m \x1b[1;34m  i\x1b[0m\x1b[48;5;236m  Ivy\x1b[0m")
    );
}
