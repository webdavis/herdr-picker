use super::*;

#[test]
fn a_workspace_herdr_has_open_wears_a_green_dot_before_its_name_on_a_plain_row_and_on_the_bar() {
    let workspaces = three_workspaces();
    let mut frame = browsing(&workspaces, 0);
    frame.rows[0].open = true;
    frame.rows[1].open = true;

    let drawn = render(&frame, LEGEND, columns(), None, 0, 0);

    assert_eq!(
        drawn.lines().nth(2),
        Some(
            "\x1b[48;5;236m\x1b[1;34m   i\x1b[22;39m\x1b[32m  ● \x1b[22;39mIvy       \x1b[2m/opt/Ivy\x1b[22;39m\x1b[0m"
        )
    );
    assert_eq!(
        drawn.lines().nth(3),
        Some(
            "\x1b[1;34m   h\x1b[22;39m\x1b[32m  ● \x1b[22;39mhomelab   \x1b[2m/opt/homelab\x1b[22;39m"
        )
    );
    assert_eq!(
        drawn.lines().nth(4),
        Some("\x1b[1;34m   f\x1b[22;39m    dotfiles  \x1b[2m/opt/dotfiles\x1b[22;39m")
    );
}

#[test]
fn the_dot_column_stays_while_a_search_blanks_the_keys() {
    let workspaces = three_workspaces();
    let mut frame = searching("ol", 2, &workspaces[1..]);
    frame.rows[1].open = true;

    assert_eq!(
        visible_rows(&render(&frame, LEGEND, columns(), None, 0, 0))[1..3],
        ["homelab   /opt/homelab", "● dotfiles  /opt/dotfiles"]
    );
}
