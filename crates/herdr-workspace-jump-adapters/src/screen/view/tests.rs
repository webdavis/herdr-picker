use super::*;
use crate::screen::fixtures::{browsing, searching, three_workspaces};

const LEGEND_ROW: &str = "\x1b[2m ctrl+n/p move  enter jump  esc close\x1b[0m";

#[test]
fn a_frame_is_the_hint_a_blank_every_row_with_a_bar_on_the_selected_one_a_blank_and_the_legend() {
    let workspaces = three_workspaces();

    assert_eq!(
        render(&browsing(&workspaces, 0, None), 20),
        format!(
            "\x1b[2m / search\x1b[0m\n\
             \n\
             \x1b[48;5;236m \x1b[1;34m  i\x1b[0m\x1b[48;5;236m  Ivy           \x1b[0m\n \
             \x1b[1;34m  h\x1b[0m  homelab\n \
             \x1b[1;34m  f\x1b[0m  dotfiles\n\
             \n\
             {LEGEND_ROW}"
        )
    );
}

#[test]
fn the_current_workspace_is_faint_and_says_so_at_the_right_edge_and_faint_on_the_bar() {
    let workspaces = three_workspaces();

    assert_eq!(
        render(&browsing(&workspaces, 0, Some(1)), 24)
            .lines()
            .nth(3),
        Some("\x1b[2m   h  homelab   current \x1b[0m")
    );
    assert_eq!(
        render(&browsing(&workspaces, 1, Some(1)), 24)
            .lines()
            .nth(3),
        Some("\x1b[48;5;236m\x1b[2m   h  homelab   current \x1b[0m")
    );
}

#[test]
fn a_width_too_narrow_for_the_row_pads_nothing_and_cuts_nothing() {
    let workspaces = three_workspaces();

    assert_eq!(
        render(&browsing(&workspaces, 0, None), 0).lines().nth(2),
        Some("\x1b[48;5;236m \x1b[1;34m  i\x1b[0m\x1b[48;5;236m  Ivy\x1b[0m")
    );
}

#[test]
fn a_search_holds_its_text_and_count_blanks_the_keys_and_parks_the_cursor_after_the_text() {
    let workspaces = three_workspaces();

    assert_eq!(
        render(&searching("ol", &workspaces[1..]), 0),
        format!(
            " /ol  \x1b[2m2/3 \x1b[0m\n\
             \n\
             \x1b[48;5;236m \x1b[1;34m   \x1b[0m\x1b[48;5;236m  homelab\x1b[0m\n \
             \x1b[1;34m   \x1b[0m  dotfiles\n\
             \n\
             {LEGEND_ROW}\x1b[1;5H"
        )
    );
}

#[test]
fn the_match_count_sits_at_the_right_edge_of_a_prompt_that_knows_its_width() {
    let workspaces = three_workspaces();

    assert_eq!(
        render(&searching("ol", &workspaces[1..]), 20)
            .lines()
            .next(),
        Some(" /ol            \x1b[2m2/3 \x1b[0m")
    );
}
