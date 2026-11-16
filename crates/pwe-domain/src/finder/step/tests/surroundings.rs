use crate::finder::fixtures::{finder_of, finder_with, press, row};
use crate::finder::{Effect, Finder, Input, step};
use crate::{Fuzzy, PickerRow, Preview};

fn previewed(label: &str) -> PickerRow {
    PickerRow {
        preview: Some(Preview::Text(format!("about {label}"))),
        ..row(label)
    }
}

fn resized(list_height: usize, preview_width: usize) -> Input {
    Input::Resized {
        list_height,
        preview_width,
    }
}

fn shown_preview(row: usize, label: &str, width: usize) -> Effect {
    Effect::ShowPreview {
        row,
        preview: Preview::Text(format!("about {label}")),
        width,
    }
}

#[test]
fn moving_to_a_row_with_a_preview_asks_for_it_at_the_preview_width() {
    let finder = finder_of(vec![previewed("one"), previewed("two")]);
    let (finder, _) = step(finder, resized(10, 30));

    let (_, effects) = step(finder, Input::Key("down".to_string()));

    assert_eq!(effects, [Effect::Draw, shown_preview(1, "two", 30)]);
}

#[test]
fn the_first_row_to_arrive_asks_for_its_preview() {
    let (_, effects) = step(
        Finder::new(Fuzzy),
        Input::RowsArrived(vec![previewed("one")]),
    );

    assert_eq!(effects, [Effect::Draw, shown_preview(0, "one", 0)]);
}

#[test]
fn a_resize_asks_for_the_preview_again_at_the_new_width() {
    let finder = finder_of(vec![previewed("one")]);

    let (_, effects) = step(finder, resized(10, 44));

    assert_eq!(effects, [Effect::Draw, shown_preview(0, "one", 44)]);
}

#[test]
fn a_row_without_a_preview_asks_for_nothing() {
    let (_, effects) = step(finder_with(&["one", "two"]), Input::Key("down".to_string()));

    assert_eq!(effects, [Effect::Draw]);
}

#[test]
fn page_down_and_page_up_move_the_cursor_by_the_list_height() {
    let labels: Vec<String> = (0..10).map(|number| format!("row {number}")).collect();
    let labels: Vec<&str> = labels.iter().map(String::as_str).collect();
    let (finder, _) = step(finder_with(&labels), resized(3, 20));

    let finder = press(finder, &["page-down", "page-down", "page-up"]);

    assert_eq!(finder.visible(3)[2].label, "row 3");
    assert!(finder.visible(3)[2].current);
}

#[test]
fn a_starting_query_narrows_the_rows_as_they_arrive() {
    let finder = Finder::new(Fuzzy).with_query("tw");
    let (finder, _) = step(finder, Input::RowsArrived(vec![row("one"), row("two")]));

    assert_eq!(finder.listed(), ["two"]);
    assert_eq!(finder.query_cursor(), 2);
}

#[test]
fn the_picker_s_header_line_is_kept_for_the_title() {
    let (finder, effects) = step(
        finder_with(&["one"]),
        Input::HeaderArrived("Pull into main?".to_string()),
    );

    assert_eq!(finder.header_line(), Some("Pull into main?"));
    assert_eq!(effects, [Effect::Draw]);
}

#[test]
fn a_failed_row_command_ends_the_rows_with_its_error() {
    let (finder, _) = step(
        finder_with(&["one"]),
        Input::RowsFailed("the picker command stopped with exit status: 2".to_string()),
    );

    assert!(finder.rows_ended());
    assert_eq!(
        finder.run_error(),
        Some("the picker command stopped with exit status: 2")
    );
}
