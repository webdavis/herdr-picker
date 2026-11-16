use super::*;
use crate::finder::Ending;
use crate::{Fuzzy, Key};

fn finder_with(rows: &[&str]) -> Finder<Fuzzy> {
    let (finder, _) = step(Finder::new(Fuzzy), arrived(rows));
    finder
}

fn arrived(rows: &[&str]) -> Input {
    Input::RowsArrived(rows.iter().map(|row| row.to_string()).collect())
}

fn press(finder: Finder<Fuzzy>, keys: &[&str]) -> Finder<Fuzzy> {
    keys.iter().fold(finder, |finder, key| {
        step(finder, Input::Key(key.to_string())).0
    })
}

fn typed(finder: Finder<Fuzzy>, text: &str) -> Finder<Fuzzy> {
    let keys: Vec<String> = text.chars().map(String::from).collect();
    let keys: Vec<&str> = keys.iter().map(String::as_str).collect();
    press(finder, &keys)
}

fn found_by_enter(finder: Finder<Fuzzy>) -> Found {
    match step(finder, Input::Key("enter".to_string())).1.as_slice() {
        [Effect::Close(found)] => found.clone(),
        effects => panic!("enter did not close the finder: {effects:?}"),
    }
}

#[test]
fn typing_narrows_the_list_and_enter_finds_the_top_row() {
    let finder = typed(finder_with(&["alpha", "beta", "bravo"]), "br");

    assert_eq!(finder.listed(), ["bravo"]);
    assert_eq!(
        found_by_enter(finder),
        Found {
            query: "br".to_string(),
            ending: Ending::Key(Key::enter()),
            rows: vec![2],
        }
    );
}

#[test]
fn matching_reads_a_label_without_its_colours() {
    let finder = typed(finder_with(&["\x1b[31mred\x1b[0m", "blue"]), "red");

    assert_eq!(finder.listed(), ["red"]);
}

#[test]
fn enter_with_marks_finds_the_marked_rows() {
    let finder = press(finder_with(&["one", "two", "three"]), &["tab", "tab"]);

    assert_eq!(found_by_enter(finder).rows, [0, 1]);
}

#[test]
fn shift_tab_marks_the_row_and_moves_up() {
    let finder = press(
        finder_with(&["one", "two", "three"]),
        &["down", "down", "shift-tab", "shift-tab"],
    );

    assert_eq!(found_by_enter(finder).rows, [2, 1]);
}

#[test]
fn tab_on_a_marked_row_unmarks_it() {
    let finder = press(finder_with(&["one", "two"]), &["tab", "up", "tab", "tab"]);

    assert_eq!(found_by_enter(finder).rows, [1]);
}

#[test]
fn marks_survive_new_rows_arriving() {
    let finder = press(finder_with(&["one", "two"]), &["down", "tab"]);
    let (finder, _) = step(finder, arrived(&["three", "four"]));

    assert_eq!(found_by_enter(finder).rows, [1]);
}

#[test]
fn marks_survive_a_query_that_hides_their_rows() {
    let finder = press(finder_with(&["one", "two"]), &["tab"]);
    let finder = typed(finder, "tw");

    assert_eq!(found_by_enter(finder).rows, [0]);
}

#[test]
fn the_cursor_stays_on_its_row_when_the_ranking_changes() {
    let finder = typed(finder_with(&["a-b-c", "x-abc-y"]), "abc");
    let finder = press(finder, &["down"]);
    let (finder, _) = step(finder, arrived(&["abc"]));

    assert_eq!(finder.listed(), ["abc", "x-abc-y", "a-b-c"]);
    assert_eq!(found_by_enter(finder).rows, [0]);
}

#[test]
fn the_cursor_stops_at_either_end_of_the_list() {
    let top = press(finder_with(&["one", "two"]), &["up"]);
    let bottom = press(finder_with(&["one", "two"]), &["down", "down", "down"]);

    assert_eq!(found_by_enter(top).rows, [0]);
    assert_eq!(found_by_enter(bottom).rows, [1]);
}

#[test]
fn enter_on_an_empty_list_finds_no_rows() {
    let finder = typed(finder_with(&["one"]), "zz");

    assert_eq!(found_by_enter(finder).rows, Vec::<usize>::new());
}

#[test]
fn the_finder_knows_when_the_rows_have_ended() {
    let finder = finder_with(&["one"]);
    assert!(!finder.rows_ended());

    let (finder, effects) = step(finder, Input::RowsEnded);

    assert!(finder.rows_ended());
    assert_eq!(effects, [Effect::Draw]);
}

#[test]
fn a_typed_space_joins_the_query() {
    let finder = press(finder_with(&["one two", "three"]), &["o", "space", "t"]);

    assert_eq!(found_by_enter(finder).query, "o t");
}
