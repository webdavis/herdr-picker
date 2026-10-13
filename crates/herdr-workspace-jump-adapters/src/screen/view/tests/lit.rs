use super::*;

#[test]
fn the_letters_a_search_matched_are_lit_in_bold_blue_on_a_plain_row_and_on_the_bar() {
    let workspaces = three_workspaces();
    let mut frame = searching("ol", 2, &workspaces[1..]);
    frame.rows[0].matched = vec![1, 4];
    frame.rows[1].matched = vec![1, 5];

    let drawn = render(&frame, &layout(), 0, 0);

    assert_eq!(
        drawn.lines().nth(2),
        Some(
            "\x1b[48;5;236m      h\x1b[1;34mo\x1b[22;39mme\x1b[1;34ml\x1b[22;39mab\x1b[38;5;239m ·· \x1b[22;39m\x1b[2m/opt/homelab\x1b[22;39m\x1b[0m"
        )
    );
    assert_eq!(
        drawn.lines().nth(3),
        Some(
            "      d\x1b[1;34mo\x1b[22;39mtfi\x1b[1;34ml\x1b[22;39mes\x1b[38;5;239m · \x1b[22;39m\x1b[2m/opt/dotfiles\x1b[22;39m"
        )
    );
}

#[test]
fn the_ellipsis_of_a_cut_name_is_never_lit() {
    let workspaces = three_workspaces();
    let mut frame = searching("hom", 3, &workspaces[1..2]);
    frame.rows[0].matched = vec![0, 1, 2, 3, 4, 5];

    let drawn = render(&frame, &layout(), 12, 0);
    let row = drawn.lines().nth(2).expect("a row");

    assert!(row.contains("\x1b[1;34mhomel\x1b[22;39m"), "{row:?}");
    assert!(row.ends_with("…\x1b[0m"), "{row:?}");
}
