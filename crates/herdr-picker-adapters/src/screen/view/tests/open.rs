use super::*;

#[test]
fn a_workspace_herdr_has_open_is_named_in_green_on_a_plain_row_and_on_the_bar() {
    let workspaces = three_workspaces();
    let mut frame = browsing(&workspaces, 0);
    frame.rows[0].open = true;
    frame.rows[1].open = true;

    let drawn = render(&frame, &layout(), 0, 0);

    assert_eq!(
        drawn.lines().nth(2),
        Some(format!(
            "{ON_BAR}{BOLD_BLUE}   i{PLAIN}  {GREEN}Ivy{PLAIN}        {GREY}/opt/Ivy{PLAIN}{END}"
        ))
        .as_deref()
    );
    assert_eq!(
        drawn.lines().nth(3),
        Some(format!(
            "{ON_BASE}{BOLD_BLUE}   h{PLAIN}  {GREEN}homelab{PLAIN}    {GREY}/opt/homelab{PLAIN}{END}"
        ))
        .as_deref()
    );
    assert_eq!(
        drawn.lines().nth(4),
        Some(format!(
            "{ON_BASE}{BOLD_BLUE}   f{PLAIN}  dotfiles   {GREY}/opt/dotfiles{PLAIN}{END}"
        ))
        .as_deref()
    );
}

#[test]
fn an_open_name_stays_green_while_a_search_blanks_the_keys_and_lights_its_letters() {
    let workspaces = three_workspaces();
    let mut frame = searching("ol", 2, &workspaces[1..]);
    frame.rows[1].open = true;
    frame.rows[1].matched = vec![1, 5];

    let drawn = render(&frame, &layout(), 0, 0);

    assert_eq!(
        drawn.lines().nth(3),
        Some(format!(
            "{ON_BASE}      {GREEN}d{PLAIN}{BOLD_BLUE}o{PLAIN}{GREEN}tfi{PLAIN}{BOLD_BLUE}l{PLAIN}{GREEN}es{PLAIN}   {GREY}/opt/dotfiles{PLAIN}{END}"
        ))
        .as_deref()
    );
}
