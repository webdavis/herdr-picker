use super::fixtures::{browsing, searching, three_workspaces};
use super::*;

#[test]
fn showing_clears_the_popup_draws_the_frame_and_going_shows_the_cursor_again() {
    let workspaces = three_workspaces();
    let frame = browsing(&workspaces, 0, None);
    let mut written = Vec::new();

    {
        let mut screen = TerminalScreen::new(&mut written, || (20, 0));
        screen.show(&frame).expect("a Vec takes every write");
    }

    assert_eq!(
        String::from_utf8(written).expect("ANSI is UTF-8"),
        format!(
            "{HIDE_CURSOR_AND_CLEAR}{}{SHOW_CURSOR}",
            view::render(&frame, 20)
        )
    );
}

#[test]
fn a_search_shows_the_cursor_in_its_prompt() {
    let workspaces = three_workspaces();
    let frame = searching("ho", &workspaces[1..2]);
    let mut written = Vec::new();

    {
        let mut screen = TerminalScreen::new(&mut written, || (20, 0));
        screen.show(&frame).expect("a Vec takes every write");
        assert!(
            screen.out.ends_with(SHOW_CURSOR.as_bytes()),
            "the prompt has no cursor"
        );
    }
}
