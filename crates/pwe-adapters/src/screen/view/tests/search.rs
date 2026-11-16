use super::*;

#[test]
fn a_search_holds_its_text_and_count_blanks_the_keys_and_parks_the_cursor_where_the_search_has_it()
{
    let workspaces = three_workspaces();

    assert_eq!(
        render(&searching("ol", 2, &workspaces[1..]), &layout(), 0, 0),
        format!(
            "{ON_BASE} /ol  {GREY}2/3 {PLAIN}{END}\n\
             {}\n\
             {ON_BAR}      homelab    {GREY}/opt/homelab{PLAIN}{END}\n\
             {ON_BASE}      dotfiles   {GREY}/opt/dotfiles{PLAIN}{END}\n\
             {}\n\
             {}\x1b[1;5H",
            blank_row(0),
            blank_row(0),
            legend_row(0)
        )
    );
    assert!(
        render(&searching("hom", 1, &workspaces[1..2]), &layout(), 0, 0).ends_with("\x1b[1;4H")
    );
}

#[test]
fn an_empty_search_hints_what_to_type_after_the_cursor_cell_and_keeps_the_count_at_the_edge() {
    let workspaces = three_workspaces();

    let drawn = render(&searching("", 0, &workspaces), &layout(), 30, 0);

    assert_eq!(
        drawn.lines().next(),
        Some(format!(
            "{ON_BASE} /{GREY} type to filter{PLAIN}         {GREY}3/3 {PLAIN}{END}"
        ))
        .as_deref()
    );
    assert!(drawn.ends_with("\x1b[1;3H"), "{drawn:?}");
    assert_eq!(
        render(&searching("", 0, &workspaces), &layout(), 0, 0)
            .lines()
            .next(),
        Some(format!(
            "{ON_BASE} /{GREY} type to filter{PLAIN}  {GREY}3/3 {PLAIN}{END}"
        ))
        .as_deref()
    );
}

#[test]
fn the_hint_is_cut_before_the_count_in_a_narrow_popup_and_goes_once_a_letter_is_typed() {
    let workspaces = three_workspaces();

    assert_eq!(
        render(&searching("", 0, &workspaces), &layout(), 12, 0)
            .lines()
            .next(),
        Some(format!(
            "{ON_BASE} /{GREY} ty…{PLAIN}  {GREY}3/3 {PLAIN}{END}"
        ))
        .as_deref()
    );
    assert_eq!(
        render(&searching("h", 1, &workspaces[1..2]), &layout(), 12, 0)
            .lines()
            .next(),
        Some(format!("{ON_BASE} /h     {GREY}1/3 {PLAIN}{END}")).as_deref()
    );
}

#[test]
fn the_match_count_sits_at_the_right_edge_of_a_prompt_that_knows_its_width() {
    let workspaces = three_workspaces();

    assert_eq!(
        render(&searching("ol", 2, &workspaces[1..]), &layout(), 20, 0)
            .lines()
            .next(),
        Some(format!("{ON_BASE} /ol            {GREY}2/3 {PLAIN}{END}")).as_deref()
    );
}

#[test]
fn a_search_nothing_matches_keeps_the_prompt_and_legend_where_they_are() {
    let frame = searching("zz", 2, &[]);

    assert_eq!(
        render(&frame, &layout(), 0, 0),
        format!(
            "{ON_BASE} /zz  {GREY}0/3 {PLAIN}{END}\n{}\n{}\n{}\x1b[1;5H",
            blank_row(0),
            blank_row(0),
            legend_row(0)
        )
    );
    assert_eq!(render(&frame, &layout(), 0, 7).lines().count(), 7);
}
