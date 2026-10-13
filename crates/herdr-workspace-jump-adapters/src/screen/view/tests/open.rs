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
        Some(
            "\x1b[48;5;236m\x1b[1;34m   i\x1b[22;39m  \x1b[32mIvy\x1b[22;39m        \x1b[2m/opt/Ivy\x1b[22;39m\x1b[0m"
        )
    );
    assert_eq!(
        drawn.lines().nth(3),
        Some(
            "\x1b[1;34m   h\x1b[22;39m  \x1b[32mhomelab\x1b[22;39m    \x1b[2m/opt/homelab\x1b[22;39m"
        )
    );
    assert_eq!(
        drawn.lines().nth(4),
        Some("\x1b[1;34m   f\x1b[22;39m  dotfiles   \x1b[2m/opt/dotfiles\x1b[22;39m")
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
        Some(
            "      \x1b[32md\x1b[22;39m\x1b[1;34mo\x1b[22;39m\x1b[32mtfi\x1b[22;39m\x1b[1;34ml\x1b[22;39m\x1b[32mes\x1b[22;39m   \x1b[2m/opt/dotfiles\x1b[22;39m"
        )
    );
}
