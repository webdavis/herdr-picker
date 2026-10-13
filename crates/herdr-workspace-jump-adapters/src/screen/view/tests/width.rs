use super::*;

#[test]
fn an_unknown_width_pads_no_row_and_cuts_nothing_but_still_aligns_the_directories() {
    let workspaces = three_workspaces();

    assert_eq!(
        render(&browsing(&workspaces, 0), &layout(), 0, 0)
            .lines()
            .nth(2),
        Some(format!(
            "{ON_BAR}{BOLD_BLUE}   i{PLAIN}  Ivy        {GREY}/opt/Ivy{PLAIN}{END}"
        ))
        .as_deref()
    );
}

#[test]
fn a_label_too_long_for_its_row_is_cut_with_an_ellipsis_and_shows_no_directory() {
    let workspaces = three_workspaces();

    assert_eq!(
        render(&browsing(&workspaces, 0), &layout(), 12, 0)
            .lines()
            .nth(3),
        Some(format!("{ON_BASE}{BOLD_BLUE}   h{PLAIN}  homel…{END}")).as_deref()
    );
}

#[test]
fn a_directory_too_long_for_what_is_left_of_its_row_is_cut_with_an_ellipsis() {
    let workspaces = three_workspaces();

    assert_eq!(
        render(&browsing(&workspaces, 0), &layout(), 23, 0)
            .lines()
            .nth(3),
        Some(format!(
            "{ON_BASE}{BOLD_BLUE}   h{PLAIN}  homelab    {GREY}/opt/…{PLAIN}{END}"
        ))
        .as_deref()
    );
}

#[test]
fn the_hint_and_the_legend_are_cut_to_a_narrow_popup() {
    let workspaces = three_workspaces();
    let drawn = render(&browsing(&workspaces, 0), &layout(), 6, 0);

    assert_eq!(
        drawn.lines().next(),
        Some(format!("{ON_BASE}{GREY} / fi…{PLAIN}{END}")).as_deref()
    );
    assert_eq!(
        drawn.lines().last(),
        Some(format!("{ON_BASE}{GREY} the …{PLAIN}{END}")).as_deref()
    );
}

#[test]
fn no_row_of_a_browsing_frame_is_wider_than_a_popup_with_room_for_its_fixed_columns() {
    let workspaces = three_workspaces();

    for width in 7..=40 {
        for line in render(&browsing(&workspaces, 2), &layout(), width, 0).lines() {
            let shown = without_escape_sequences(line).chars().count();
            assert!(shown <= width, "{shown} columns in {width}: {line:?}");
        }
    }
}
