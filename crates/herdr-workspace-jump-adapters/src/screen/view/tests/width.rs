use super::*;

#[test]
fn an_unknown_width_fills_no_bar_and_cuts_nothing_but_still_aligns_the_directories() {
    let workspaces = three_workspaces();

    assert_eq!(
        render(&browsing(&workspaces, 0), LEGEND, columns(), None, 0, 0)
            .lines()
            .nth(2),
        Some(
            "\x1b[48;5;236m \x1b[1;34m  i\x1b[0m\x1b[48;5;236m   \x1b[0m\x1b[48;5;236m Ivy       \x1b[0m\x1b[48;5;236m\x1b[2m/opt/Ivy\x1b[0m"
        )
    );
}

#[test]
fn a_label_too_long_for_its_row_is_cut_with_an_ellipsis_and_shows_no_directory() {
    let workspaces = three_workspaces();

    assert_eq!(
        render(&browsing(&workspaces, 0), LEGEND, columns(), None, 12, 0)
            .lines()
            .nth(3),
        Some(" \x1b[1;34m  h\x1b[0m    hom…")
    );
}

#[test]
fn a_directory_too_long_for_what_is_left_of_its_row_is_cut_with_an_ellipsis() {
    let workspaces = three_workspaces();

    assert_eq!(
        render(&browsing(&workspaces, 0), LEGEND, columns(), None, 23, 0)
            .lines()
            .nth(3),
        Some(" \x1b[1;34m  h\x1b[0m    homelab   \x1b[2m/opt…\x1b[0m")
    );
}

#[test]
fn the_hint_and_the_legend_are_cut_to_a_narrow_popup() {
    let workspaces = three_workspaces();
    let drawn = render(&browsing(&workspaces, 0), LEGEND, columns(), None, 6, 0);

    assert_eq!(drawn.lines().next(), Some("\x1b[2m / fi…\x1b[0m"));
    assert_eq!(drawn.lines().last(), Some("\x1b[2m the …\x1b[0m"));
}

#[test]
fn no_row_of_a_browsing_frame_is_wider_than_a_popup_with_room_for_its_fixed_columns() {
    let workspaces = three_workspaces();

    for width in 7..=40 {
        for line in render(&browsing(&workspaces, 2), LEGEND, columns(), None, width, 0).lines() {
            let shown = without_escape_sequences(line).chars().count();
            assert!(shown <= width, "{shown} columns in {width}: {line:?}");
        }
    }
}
