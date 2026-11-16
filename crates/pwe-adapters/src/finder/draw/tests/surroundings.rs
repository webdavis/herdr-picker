use pwe_domain::finder::{Finder, Input, step};
use pwe_domain::{Fuzzy, Theme};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::layout::{Position, Rect};
use ratatui::style::Color;

use super::{after, finder_of, key, lines, row};
use crate::finder::{draw, resized};

#[test]
fn the_picker_s_header_line_shows_under_the_title() {
    let header = Input::HeaderArrived("Pull into main?".to_string());
    let finder = after(finder_of(vec![row("one")]), vec![header]);

    let lines = lines(&super::drawn(&finder, 40, 8));

    assert_eq!(lines[1], "Files               │");
    assert_eq!(lines[2], "Pull into main?     │");
    assert_eq!(lines[3], "  one               │");
}

#[test]
fn a_failed_row_command_shows_its_error_in_red_in_the_footer() {
    let failed = Input::RowsFailed("exit status: 2".to_string());
    let finder = step(Finder::new(Fuzzy), failed).0;

    let buffer = super::drawn(&finder, 40, 6);

    let lines = lines(&buffer);
    let error_line = lines
        .iter()
        .position(|line| line.starts_with("exit status: 2"))
        .expect("the error in the footer");
    assert_eq!(buffer[(0, error_line as u16)].fg, Color::Rgb(243, 139, 168));
}

#[test]
fn the_cursor_stands_in_the_search_box_where_the_query_is_edited() {
    let finder = after(
        finder_of(vec![row("one")]),
        vec![key("o"), key("n"), key("left")],
    );
    let mut terminal = Terminal::new(TestBackend::new(40, 6)).expect("a test terminal");

    terminal
        .draw(|frame| draw(frame, &finder, Theme::default()))
        .expect("a drawn frame");

    assert_eq!(
        terminal.get_cursor_position().expect("a cursor"),
        Position::new(3, 0)
    );
}

#[test]
fn a_resize_reports_the_list_height_and_the_preview_width() {
    assert_eq!(
        resized(Rect::new(0, 0, 80, 24)),
        Input::Resized {
            list_height: 20,
            preview_width: 39
        }
    );
}
