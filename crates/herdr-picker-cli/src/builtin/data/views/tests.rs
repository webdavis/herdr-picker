use herdr_picker_domain::Action;

use crate::builtin::data::tests::{ACCOUNT, DIM, RESET, file_rows, holding, next};
use crate::builtin::git::fixture::labels;

const BLUE: &str = "\x1b[38;2;137;180;250m";

#[test]
fn levels_shows_the_top_keys_and_enter_on_an_object_opens_the_level_below() {
    let tree = holding("data-levels", "a.json", ACCOUNT);
    let file = tree.path("a.json");

    let (header, rows) = file_rows(&tree, "a.json", "levels", None);

    assert_eq!(header, Some(".".to_string()));
    assert_eq!(
        labels(&rows),
        vec![
            format!("{BLUE}workspace/{RESET}  {DIM}{{1 key}}{RESET}"),
            format!("password  {DIM}\"hunter2\"{RESET}"),
            format!("{BLUE}payload/{RESET}  {DIM}{{1 key}}{RESET}  {DIM}decoded{RESET}"),
        ]
    );
    assert_eq!(
        rows[0].enter,
        next("data-file", &file, Some(r#"["workspace"]"#))
    );
    assert_eq!(rows[1].enter, Action::Insert(".password".to_string()));

    let (header, rows) = file_rows(
        &tree,
        "a.json",
        "levels",
        Some(r#"["workspace","ignoreGlobs"]"#),
    );

    assert_eq!(header, Some(".workspace.ignoreGlobs".to_string()));
    assert_eq!(
        labels(&rows),
        vec![
            format!("[0]  {DIM}\"*.log\"{RESET}"),
            format!("[1]  {DIM}\"dist\"{RESET}")
        ]
    );
}

#[test]
fn zoom_lists_everything_inside_the_object_it_opened_on() {
    let tree = holding("data-zoom", "a.json", ACCOUNT);
    let file = tree.path("a.json");

    let (header, rows) = file_rows(&tree, "a.json", "zoom", Some(r#"["workspace"]"#));

    assert_eq!(
        header,
        Some(".workspace  Enter on an object or array zooms in; Esc comes back.".to_string())
    );
    assert_eq!(
        labels(&rows),
        vec![
            format!(".workspace  {DIM}{{1 key}}{RESET}"),
            format!(".workspace.ignoreGlobs  {DIM}[2 items]{RESET}"),
            format!(".workspace.ignoreGlobs[0]  {DIM}\"*.log\"{RESET}"),
            format!(".workspace.ignoreGlobs[1]  {DIM}\"dist\"{RESET}"),
        ]
    );
    assert_eq!(
        rows[1].enter,
        next("data-file", &file, Some(r#"["workspace","ignoreGlobs"]"#))
    );
    assert_eq!(
        rows[2].enter,
        Action::Insert(".workspace.ignoreGlobs[0]".to_string())
    );
}

#[test]
fn keys_lists_every_place_a_key_appears_the_places_of_one_key_together() {
    let tree = holding(
        "data-keys",
        "a.json",
        r#"{"id": 1, "user": {"id": 2, "name": "ann"}}"#,
    );

    let (_, rows) = file_rows(&tree, "a.json", "keys", None);

    assert_eq!(
        labels(&rows),
        vec![
            format!("id  .id  {DIM}1{RESET}"),
            format!("id  .user.id  {DIM}2{RESET}"),
            format!("name  .user.name  {DIM}\"ann\"{RESET}"),
            format!("user  .user  {DIM}{{2 keys}}{RESET}"),
        ]
    );
    assert_eq!(rows[1].enter, Action::Insert(".user.id".to_string()));
}

#[test]
fn shape_shows_each_field_once_with_the_first_value_as_its_example() {
    let tree = holding(
        "data-shape",
        "a.json",
        r#"{"users": [{"name": "ann", "age": 30}, {"name": "bo", "email": "b@x"}]}"#,
    );

    let (header, rows) = file_rows(&tree, "a.json", "shape", None);

    assert_eq!(header, Some(super::SHAPE_NOTE.to_string()));
    assert_eq!(
        labels(&rows),
        vec![
            format!(".  {DIM}{{1 key}}{RESET}"),
            format!(".users  {DIM}[2 items]{RESET}"),
            format!(".users[]  {DIM}{{2 keys}}{RESET}"),
            format!(".users[].name  {DIM}\"ann\"{RESET}"),
            format!(".users[].age  {DIM}30{RESET}"),
            format!(".users[].email  {DIM}\"b@x\"{RESET}"),
        ]
    );
    assert_eq!(rows[3].enter, Action::Insert(".users[].name".to_string()));
}
