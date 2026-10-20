use super::*;
use crate::picker_key::Key;
use crate::picker_row::{Location, PickerCall};

fn row() -> PickerRow {
    let key = |name| Key::parse(name).expect("a key");
    PickerRow {
        label: "notes.txt".to_string(),
        preview: None,
        enter: Action::Insert("notes.txt".to_string()),
        keys: vec![
            (
                key("ctrl-enter"),
                Action::Edit(vec![Location {
                    path: "notes.txt".to_string(),
                    line: Some(3),
                }]),
            ),
            (key("alt-y"), Action::CopyStay("notes.txt".to_string())),
            (
                key("ctrl-o"),
                Action::Next(PickerCall {
                    name: "git-log".to_string(),
                    args: Vec::new(),
                    state: None,
                    query: None,
                }),
            ),
            (key("ctrl-x"), Action::Nothing),
        ],
        group: None,
    }
}

#[test]
fn the_footer_names_each_key_of_the_highlighted_row_then_help() {
    assert_eq!(
        footer_hint(&row()),
        vec![
            "enter insert",
            "ctrl-enter edit",
            "alt-y copy and stay",
            "ctrl-o go to",
            "F1 help"
        ]
    );
}

#[test]
fn help_lists_the_rows_keys_in_detail_then_the_keys_every_picker_has() {
    let entry = |key: &str, text: &str| (key.to_string(), text.to_string());

    assert_eq!(
        help_entries(Some(&row()), true),
        vec![
            entry("enter", "insert notes.txt"),
            entry("ctrl-enter", "edit notes.txt:3"),
            entry("alt-y", "copy and stay notes.txt"),
            entry("ctrl-o", "go to Git Log"),
            entry("ctrl-space", "choose a source"),
            entry("tab", "mark a row; a key then acts on every marked row"),
            entry(
                "ctrl-q",
                "save the marked rows' files and lines as the quickfix list"
            ),
            entry(
                "alt-o",
                "send the marked rows to an agent, copy them, or ask a quick question about them"
            ),
            entry("esc", "back to the previous step, or close on the first"),
            entry("F1", "this help"),
        ]
    );
    assert_eq!(help_entries(None, false).len(), 5);
}

#[test]
fn a_source_key_reads_as_show_and_names_the_source() {
    let toggle = PickerRow {
        label: "notes.txt".to_string(),
        preview: None,
        enter: Action::Nothing,
        keys: vec![(
            Key::parse("alt-.").expect("a key"),
            Action::Source("hidden".to_string()),
        )],
        group: None,
    };

    assert_eq!(footer_hint(&toggle), vec!["alt-. show", "F1 help"]);
    assert_eq!(
        help_entries(Some(&toggle), false)[0],
        ("alt-.".to_string(), "show hidden".to_string())
    );
}

#[test]
fn a_state_key_reads_as_switch() {
    let toggle = PickerRow {
        label: "a.rs:1: x".to_string(),
        preview: None,
        enter: Action::Nothing,
        keys: vec![(
            Key::parse("alt-x").expect("a key"),
            Action::State("{\"regex\":true}".to_string()),
        )],
        group: None,
    };

    assert_eq!(footer_hint(&toggle), vec!["alt-x switch", "F1 help"]);
}
