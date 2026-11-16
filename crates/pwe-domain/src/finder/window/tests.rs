use crate::finder::fixtures::{finder_of, finder_with, press, row, typed};
use crate::finder::{Input, ListLine, step};
use crate::{PickerRow, Preview};

fn preview_arrived(row: usize, text: &str) -> Input {
    Input::PreviewArrived {
        row,
        text: text.to_string(),
    }
}

#[test]
fn a_listed_line_keeps_the_row_s_colours_and_matches_its_plain_text() {
    let finder = typed(finder_of(vec![row("\x1b[31mred\x1b[0m")]), "rd");

    let lines = finder.visible(5);

    assert_eq!(lines[0].label, "\x1b[31mred\x1b[0m");
    assert_eq!(lines[0].matched, [0, 2]);
}

#[test]
fn the_window_keeps_the_cursor_in_sight_and_shows_marks() {
    let finder = press(finder_with(&["one", "two", "three"]), &["tab", "down"]);

    let lines: Vec<(&str, bool, bool)> = finder
        .visible(2)
        .iter()
        .map(|line: &ListLine| (line.label, line.marked, line.current))
        .collect();

    assert_eq!(lines, [("two", false, false), ("three", false, true)]);
    assert!(finder.visible(3)[0].marked);
}

#[test]
fn a_preview_for_the_current_row_is_shown() {
    let (finder, _) = step(finder_with(&["one", "two"]), preview_arrived(0, "first"));

    assert_eq!(finder.preview(), Some("first"));
}

#[test]
fn a_preview_for_a_row_the_cursor_left_is_dropped() {
    let finder = press(finder_with(&["one", "two"]), &["down"]);
    let (finder, _) = step(finder, preview_arrived(0, "first"));

    assert_eq!(finder.preview(), None);
}

#[test]
fn a_preview_is_hidden_once_the_cursor_moves_on() {
    let (finder, _) = step(finder_with(&["one", "two"]), preview_arrived(0, "first"));

    assert_eq!(press(finder, &["down"]).preview(), None);
}

#[test]
fn the_current_row_is_the_one_under_the_cursor() {
    let second = PickerRow {
        preview: Some(Preview::Text("t".to_string())),
        ..row("two")
    };
    let finder = press(finder_of(vec![row("one"), second.clone()]), &["down"]);

    assert_eq!(finder.current_row(), Some(&second));
}
