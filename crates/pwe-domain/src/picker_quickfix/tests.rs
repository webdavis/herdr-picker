use super::*;
use crate::picker_key::Key;
use crate::picker_row::{Action, Location};

fn row(label: &str, enter: Action, keys: Vec<(Key, Action)>) -> PickerRow {
    PickerRow {
        label: label.to_string(),
        preview: None,
        enter,
        keys,
        group: None,
    }
}

fn at(path: &str, line: Option<u32>) -> Action {
    Action::Edit(vec![Location {
        path: path.to_string(),
        line,
    }])
}

#[test]
fn each_marked_row_that_edits_a_file_is_one_entry_at_its_line_named_by_its_plain_label() {
    let error = row(
        "\x1b[31merror\x1b[0m: oops",
        at("/p/src/a.rs", Some(7)),
        Vec::new(),
    );
    let file = row(
        "b.txt",
        Action::Insert("b.txt".to_string()),
        vec![(
            Key::parse("ctrl-enter").expect("a key"),
            at("/p/b.txt", None),
        )],
    );
    let note = row("no file here", Action::Copy("x".to_string()), Vec::new());

    assert_eq!(
        quickfix_entries(&[&error, &note, &file]),
        vec![
            QuickfixEntry {
                path: "/p/src/a.rs".to_string(),
                line: 7,
                text: "error: oops".to_string(),
            },
            QuickfixEntry {
                path: "/p/b.txt".to_string(),
                line: 1,
                text: "b.txt".to_string(),
            },
        ]
    );
}

#[test]
fn rows_without_a_file_make_no_entries() {
    let note = row("no file here", Action::Nothing, Vec::new());

    assert_eq!(quickfix_entries(&[&note]), Vec::new());
}
