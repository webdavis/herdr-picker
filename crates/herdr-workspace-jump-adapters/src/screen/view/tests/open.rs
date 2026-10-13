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
            "\x1b[48;2;54;54;70m\x1b[1;38;2;137;180;250m   i\x1b[22;39m  \x1b[38;2;166;227;161mIvy\x1b[22;39m        \x1b[38;2;127;132;156m/opt/Ivy\x1b[22;39m\x1b[0m"
        )
    );
    assert_eq!(
        drawn.lines().nth(3),
        Some(
            "\x1b[1;38;2;137;180;250m   h\x1b[22;39m  \x1b[38;2;166;227;161mhomelab\x1b[22;39m    \x1b[38;2;127;132;156m/opt/homelab\x1b[22;39m"
        )
    );
    assert_eq!(
        drawn.lines().nth(4),
        Some(
            "\x1b[1;38;2;137;180;250m   f\x1b[22;39m  dotfiles   \x1b[38;2;127;132;156m/opt/dotfiles\x1b[22;39m"
        )
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
            "      \x1b[38;2;166;227;161md\x1b[22;39m\x1b[1;38;2;137;180;250mo\x1b[22;39m\x1b[38;2;166;227;161mtfi\x1b[22;39m\x1b[1;38;2;137;180;250ml\x1b[22;39m\x1b[38;2;166;227;161mes\x1b[22;39m   \x1b[38;2;127;132;156m/opt/dotfiles\x1b[22;39m"
        )
    );
}
