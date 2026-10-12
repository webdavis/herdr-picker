use super::*;
use crate::screen::fixtures::{browsing, searching, three_workspaces};

const LEGEND_ROW: &str = "\x1b[2m ctrl+n/p move  enter jump  esc close\x1b[0m";

fn visible_rows(drawn: &str) -> Vec<String> {
    drawn
        .lines()
        .map(without_escape_sequences)
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
        .collect()
}

fn without_escape_sequences(line: &str) -> String {
    let mut text = String::new();
    let mut rest = line;
    while let Some(start) = rest.find('\x1b') {
        text.push_str(&rest[..start]);
        let sequence = &rest[start..];
        rest = sequence
            .find(ends_an_escape_sequence)
            .map_or("", |end| &sequence[end + 1..]);
    }
    text.push_str(rest);
    text
}

fn ends_an_escape_sequence(character: char) -> bool {
    character.is_ascii_alphabetic()
}

#[test]
fn a_frame_is_the_hint_a_blank_every_row_with_a_bar_on_the_selected_one_a_blank_and_the_legend() {
    let workspaces = three_workspaces();

    assert_eq!(
        render(&browsing(&workspaces, 0, None), 20, 0),
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
fn a_known_height_is_filled_exactly_with_the_legend_on_its_last_row() {
    let workspaces = three_workspaces();
    let frame = browsing(&workspaces, 0, None);

    assert_eq!(render(&frame, 20, 7), render(&frame, 20, 0));
    let taller = render(&frame, 20, 9);
    assert_eq!(taller.lines().count(), 9);
    assert_eq!(taller.lines().nth(5), Some(""));
    assert_eq!(taller.lines().last(), Some(LEGEND_ROW));
    assert!(
        !taller.ends_with('\n'),
        "a newline after the legend scrolls a full popup up a row"
    );
}

#[test]
fn a_list_taller_than_its_space_scrolls_to_keep_the_bar_in_view() {
    let workspaces = three_workspaces();

    assert_eq!(
        visible_rows(&render(&browsing(&workspaces, 0, None), 0, 6)),
        [
            "/ search",
            "i  Ivy",
            "h  homelab",
            "ctrl+n/p move  enter jump  esc close"
        ]
    );
    let scrolled = render(&browsing(&workspaces, 2, None), 0, 6);
    assert_eq!(
        visible_rows(&scrolled),
        [
            "/ search",
            "h  homelab",
            "f  dotfiles",
            "ctrl+n/p move  enter jump  esc close"
        ]
    );
    assert!(
        scrolled.contains("\x1b[48;5;236m \x1b[1;34m  f"),
        "the bar did not follow the selected row"
    );
}

#[test]
fn the_current_workspace_is_faint_and_says_so_at_the_right_edge_and_faint_on_the_bar() {
    let workspaces = three_workspaces();

    assert_eq!(
        render(&browsing(&workspaces, 0, Some(1)), 24, 0)
            .lines()
            .nth(3),
        Some("\x1b[2m   h  homelab   current \x1b[0m")
    );
    assert_eq!(
        render(&browsing(&workspaces, 1, Some(1)), 24, 0)
            .lines()
            .nth(3),
        Some("\x1b[48;5;236m\x1b[2m   h  homelab   current \x1b[0m")
    );
}

#[test]
fn a_width_too_narrow_for_the_row_pads_nothing_and_cuts_nothing() {
    let workspaces = three_workspaces();

    assert_eq!(
        render(&browsing(&workspaces, 0, None), 0, 0).lines().nth(2),
        Some("\x1b[48;5;236m \x1b[1;34m  i\x1b[0m\x1b[48;5;236m  Ivy\x1b[0m")
    );
}

#[test]
fn a_search_holds_its_text_and_count_blanks_the_keys_and_parks_the_cursor_where_the_search_has_it()
{
    let workspaces = three_workspaces();

    assert_eq!(
        render(&searching("ol", 2, &workspaces[1..]), 0, 0),
        format!(
            " /ol  \x1b[2m2/3 \x1b[0m\n\
             \n\
             \x1b[48;5;236m \x1b[1;34m   \x1b[0m\x1b[48;5;236m  homelab\x1b[0m\n \
             \x1b[1;34m   \x1b[0m  dotfiles\n\
             \n\
             {LEGEND_ROW}\x1b[1;5H"
        )
    );
    assert!(render(&searching("hom", 1, &workspaces[1..2]), 0, 0).ends_with("\x1b[1;4H"));
}

#[test]
fn the_match_count_sits_at_the_right_edge_of_a_prompt_that_knows_its_width() {
    let workspaces = three_workspaces();

    assert_eq!(
        render(&searching("ol", 2, &workspaces[1..]), 20, 0)
            .lines()
            .next(),
        Some(" /ol            \x1b[2m2/3 \x1b[0m")
    );
}

#[test]
fn a_search_nothing_matches_keeps_the_prompt_and_legend_where_they_are() {
    let frame = searching("zz", 2, &[]);

    assert_eq!(
        render(&frame, 0, 0),
        format!(" /zz  \x1b[2m0/3 \x1b[0m\n\n\n{LEGEND_ROW}\x1b[1;5H")
    );
    assert_eq!(render(&frame, 0, 7).lines().count(), 7);
}
