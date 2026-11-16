use super::*;

#[test]
fn a_frame_is_the_hint_a_blank_every_row_with_its_directory_and_a_bar_on_the_selected_one_a_blank_and_the_legend()
 {
    let workspaces = three_workspaces();

    assert_eq!(
        render(&browsing(&workspaces, 0), &layout(), 32, 0),
        format!(
            "{ON_BASE}{GREY} / find{PLAIN}                         {END}\n\
             {}\n\
             {ON_BAR}{BOLD_BLUE}   i{PLAIN}  Ivy        {GREY}/opt/Ivy{PLAIN}       {END}\n\
             {ON_BASE}{BOLD_BLUE}   h{PLAIN}  homelab    {GREY}/opt/homelab{PLAIN}   {END}\n\
             {ON_BASE}{BOLD_BLUE}   f{PLAIN}  dotfiles   {GREY}/opt/dotfiles{PLAIN}  {END}\n\
             {}\n\
             {}",
            blank_row(32),
            blank_row(32),
            legend_row(32)
        )
    );
}

#[test]
fn the_directory_column_starts_after_the_widest_name_whatever_rows_are_shown() {
    let workspaces = three_workspaces();

    let only_ivy = render(&browsing(&workspaces[..1], 0), &layout(), 0, 0);

    assert_eq!(visible_rows(&only_ivy)[1], "i  Ivy        /opt/Ivy");
}

#[test]
fn a_known_height_is_filled_exactly_with_the_legend_on_its_last_row() {
    let workspaces = three_workspaces();
    let frame = browsing(&workspaces, 0);

    assert_eq!(
        render(&frame, &layout(), 32, 7),
        render(&frame, &layout(), 32, 0)
    );
    let taller = render(&frame, &layout(), 32, 9);
    assert_eq!(taller.lines().count(), 9);
    assert_eq!(taller.lines().nth(5), Some(blank_row(32).as_str()));
    assert_eq!(taller.lines().last(), Some(legend_row(32).as_str()));
    assert!(
        !taller.ends_with('\n'),
        "a newline after the legend scrolls a full popup up a row"
    );
}

#[test]
fn a_list_taller_than_its_space_scrolls_to_keep_the_bar_in_view() {
    let workspaces = three_workspaces();

    assert_eq!(
        visible_rows(&render(&browsing(&workspaces, 0), &layout(), 0, 6)),
        [
            "/ find",
            "i  Ivy        /opt/Ivy",
            "h  homelab    /opt/homelab",
            "the keys"
        ]
    );
    let scrolled = render(&browsing(&workspaces, 2), &layout(), 0, 6);
    assert_eq!(
        visible_rows(&scrolled),
        [
            "/ find",
            "h  homelab    /opt/homelab",
            "f  dotfiles   /opt/dotfiles",
            "the keys"
        ]
    );
    assert!(
        scrolled.contains(&format!("{ON_BAR}{BOLD_BLUE}   f")),
        "the bar did not follow the selected row"
    );
}
