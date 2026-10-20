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
        Some(format!(
            "{ON_BAR}      h{BOLD_BLUE}o{PLAIN}me{BOLD_BLUE}l{PLAIN}ab    {GREY}/opt/homelab{PLAIN}{END}"
        ))
        .as_deref()
    );
    assert_eq!(
        drawn.lines().nth(3),
        Some(format!(
            "{ON_BASE}      d{BOLD_BLUE}o{PLAIN}tfi{BOLD_BLUE}l{PLAIN}es   {GREY}/opt/dotfiles{PLAIN}{END}"
        ))
        .as_deref()
    );
}

#[test]
fn the_ellipsis_of_a_cut_name_is_never_lit() {
    let workspaces = three_workspaces();
    let mut frame = searching("hom", 3, &workspaces[1..2]);
    frame.rows[0].matched = vec![0, 1, 2, 3, 4, 5];

    let drawn = render(&frame, &layout(), 12, 0);
    let row = drawn.lines().nth(2).expect("a row");

    assert!(row.contains(&format!("{BOLD_BLUE}homel{PLAIN}")), "{row:?}");
    assert!(row.ends_with(&format!("…{END}")), "{row:?}");
}
