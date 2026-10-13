use super::*;

#[test]
fn a_search_holds_its_text_and_count_blanks_the_keys_and_parks_the_cursor_where_the_search_has_it()
{
    let workspaces = three_workspaces();

    assert_eq!(
        render(
            &searching("ol", 2, &workspaces[1..]),
            LEGEND,
            columns(),
            None,
            0,
            0
        ),
        format!(
            " /ol  \x1b[2m2/3 \x1b[0m\n\
             \n\
             \x1b[48;5;236m \x1b[1;34m   \x1b[0m\x1b[48;5;236m   \x1b[0m\x1b[48;5;236m homelab   \x1b[0m\x1b[48;5;236m\x1b[2m/opt/homelab\x1b[0m\n \
             \x1b[1;34m   \x1b[0m    dotfiles  \x1b[2m/opt/dotfiles\x1b[0m\n\
             \n\
             {LEGEND_ROW}\x1b[1;5H"
        )
    );
    assert!(
        render(
            &searching("hom", 1, &workspaces[1..2]),
            LEGEND,
            columns(),
            None,
            0,
            0
        )
        .ends_with("\x1b[1;4H")
    );
}

#[test]
fn an_empty_search_hints_what_to_type_after_the_cursor_cell_and_keeps_the_count_at_the_edge() {
    let workspaces = three_workspaces();

    let drawn = render(
        &searching("", 0, &workspaces),
        LEGEND,
        columns(),
        None,
        30,
        0,
    );

    assert_eq!(
        drawn.lines().next(),
        Some(" /\x1b[2m type to filter\x1b[0m         \x1b[2m3/3 \x1b[0m")
    );
    assert!(drawn.ends_with("\x1b[1;3H"), "{drawn:?}");
    assert_eq!(
        render(
            &searching("", 0, &workspaces),
            LEGEND,
            columns(),
            None,
            0,
            0
        )
        .lines()
        .next(),
        Some(" /\x1b[2m type to filter\x1b[0m  \x1b[2m3/3 \x1b[0m")
    );
}

#[test]
fn the_hint_is_cut_before_the_count_in_a_narrow_popup_and_goes_once_a_letter_is_typed() {
    let workspaces = three_workspaces();

    assert_eq!(
        render(
            &searching("", 0, &workspaces),
            LEGEND,
            columns(),
            None,
            12,
            0
        )
        .lines()
        .next(),
        Some(" /\x1b[2m ty…\x1b[0m  \x1b[2m3/3 \x1b[0m")
    );
    assert_eq!(
        render(
            &searching("h", 1, &workspaces[1..2]),
            LEGEND,
            columns(),
            None,
            12,
            0
        )
        .lines()
        .next(),
        Some(" /h     \x1b[2m1/3 \x1b[0m")
    );
}

#[test]
fn the_match_count_sits_at_the_right_edge_of_a_prompt_that_knows_its_width() {
    let workspaces = three_workspaces();

    assert_eq!(
        render(
            &searching("ol", 2, &workspaces[1..]),
            LEGEND,
            columns(),
            None,
            20,
            0
        )
        .lines()
        .next(),
        Some(" /ol            \x1b[2m2/3 \x1b[0m")
    );
}

#[test]
fn a_search_nothing_matches_keeps_the_prompt_and_legend_where_they_are() {
    let frame = searching("zz", 2, &[]);

    assert_eq!(
        render(&frame, LEGEND, columns(), None, 0, 0),
        format!(" /zz  \x1b[2m0/3 \x1b[0m\n\n\n{LEGEND_ROW}\x1b[1;5H")
    );
    assert_eq!(
        render(&frame, LEGEND, columns(), None, 0, 7)
            .lines()
            .count(),
        7
    );
}
