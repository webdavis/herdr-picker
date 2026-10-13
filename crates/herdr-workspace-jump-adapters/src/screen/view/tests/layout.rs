use super::*;

#[test]
fn a_frame_is_the_hint_a_blank_every_row_with_its_directory_and_a_bar_on_the_selected_one_a_blank_and_the_legend()
 {
    let workspaces = three_workspaces();

    assert_eq!(
        render(&browsing(&workspaces, 0), LEGEND, columns(), None, 32, 0),
        format!(
            "\x1b[2m / find\x1b[0m\n\
             \n\
             \x1b[48;5;236m\x1b[1;34m   i\x1b[22;39m    Ivy       \x1b[2m/opt/Ivy\x1b[22;39m      \x1b[0m\n\
             \x1b[1;34m   h\x1b[22;39m    homelab   \x1b[2m/opt/homelab\x1b[22;39m\n\
             \x1b[1;34m   f\x1b[22;39m    dotfiles  \x1b[2m/opt/dotfiles\x1b[22;39m\n\
             \n\
             {LEGEND_ROW}"
        )
    );
}

#[test]
fn the_directory_column_starts_after_the_widest_name_whatever_rows_are_shown() {
    let workspaces = three_workspaces();

    let only_ivy = render(
        &browsing(&workspaces[..1], 0),
        LEGEND,
        columns(),
        None,
        0,
        0,
    );

    assert_eq!(visible_rows(&only_ivy)[1], "i    Ivy       /opt/Ivy");
}

#[test]
fn a_known_height_is_filled_exactly_with_the_legend_on_its_last_row() {
    let workspaces = three_workspaces();
    let frame = browsing(&workspaces, 0);

    assert_eq!(
        render(&frame, LEGEND, columns(), None, 32, 7),
        render(&frame, LEGEND, columns(), None, 32, 0)
    );
    let taller = render(&frame, LEGEND, columns(), None, 32, 9);
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
        visible_rows(&render(
            &browsing(&workspaces, 0),
            LEGEND,
            columns(),
            None,
            0,
            6
        )),
        [
            "/ find",
            "i    Ivy       /opt/Ivy",
            "h    homelab   /opt/homelab",
            "the keys"
        ]
    );
    let scrolled = render(&browsing(&workspaces, 2), LEGEND, columns(), None, 0, 6);
    assert_eq!(
        visible_rows(&scrolled),
        [
            "/ find",
            "h    homelab   /opt/homelab",
            "f    dotfiles  /opt/dotfiles",
            "the keys"
        ]
    );
    assert!(
        scrolled.contains("\x1b[48;5;236m\x1b[1;34m   f"),
        "the bar did not follow the selected row"
    );
}
