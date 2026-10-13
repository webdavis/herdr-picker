use super::*;

#[test]
fn the_letters_a_search_matched_are_lit_in_bold_blue_on_a_plain_row_and_on_the_bar() {
    let workspaces = three_workspaces();
    let mut frame = searching("ol", 2, &workspaces[1..]);
    frame.rows[0].matched = vec![1, 4];
    frame.rows[1].matched = vec![1, 5];

    let drawn = render(&frame, LEGEND, columns(), 0, 0);

    assert_eq!(
        drawn.lines().nth(2),
        Some(
            "\x1b[48;5;236m \x1b[1;34m   \x1b[0m\x1b[48;5;236m   \x1b[0m\x1b[48;5;236m h\x1b[0m\x1b[48;5;236m\x1b[1;34mo\x1b[0m\x1b[48;5;236mme\x1b[0m\x1b[48;5;236m\x1b[1;34ml\x1b[0m\x1b[48;5;236mab   \x1b[0m\x1b[48;5;236m\x1b[2m/opt/homelab\x1b[0m"
        )
    );
    assert_eq!(
        drawn.lines().nth(3),
        Some(
            " \x1b[1;34m   \x1b[0m    d\x1b[1;34mo\x1b[0mtfi\x1b[1;34ml\x1b[0mes  \x1b[2m/opt/dotfiles\x1b[0m"
        )
    );
}

#[test]
fn the_ellipsis_of_a_cut_name_is_never_lit() {
    let workspaces = three_workspaces();
    let mut frame = searching("hom", 3, &workspaces[1..2]);
    frame.rows[0].matched = vec![0, 1, 2, 3];

    let drawn = render(&frame, LEGEND, columns(), 12, 0);
    let row = drawn.lines().nth(2).expect("a row");

    assert!(row.contains("\x1b[1;34mhom\x1b[0m"), "{row:?}");
    assert!(row.ends_with("\x1b[48;5;236m…\x1b[0m"), "{row:?}");
}
