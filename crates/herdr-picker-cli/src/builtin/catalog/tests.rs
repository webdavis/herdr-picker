use std::collections::BTreeMap;

use herdr_picker_domain::{PickerCall, PickerCommand, PickerDeclaration, Preview, pickers};

use super::*;

const DIM: &str = "\u{1b}[2m";
const RESET: &str = "\u{1b}[0m";

const CHORD_TABLE: &str = "# key\tgroup\tkind\taction\tdescription
ctrl-a\tediting\tcommand\tbeginning-of-line\t
alt-l\tlisting\trun\teza -ahlrs date\tls/eza
alt-L\tlisting\tinsert\teza -ahlrs date\t
ctrl-s\tsudo\tfunction\tsudo_insert\tInsert 'sudo ' at the beginning of a command.

";

fn declared(name: &str, key: &str, description: &str, command: PickerCommand) -> PickerDeclaration {
    PickerDeclaration {
        name: name.to_string(),
        key: key.to_string(),
        description: description.to_string(),
        command,
        args: Vec::new(),
        title: None,
        sources: Vec::new(),
        steps: BTreeMap::new(),
        live: false,
    }
}

fn some_pickers() -> Vec<Picker> {
    let builtin = |name: &str| PickerCommand::Builtin(name.to_string());
    pickers([
        declared(
            "notes",
            "ctrl-t m",
            "Notes in the vault",
            PickerCommand::Program(vec!["/p/notes.py".to_string()]),
        ),
        declared(
            "bookmark",
            "ctrl-t ctrl-b",
            "Bookmark this folder",
            builtin("bookmark"),
        ),
        declared("pickers", "ctrl-t ;", "Every picker", builtin("pickers")),
    ])
    .expect("pickers")
}

fn start(name: &str) -> Action {
    Action::Next(PickerCall {
        name: name.to_string(),
        args: Vec::new(),
        state: None,
        query: None,
    })
}

fn row(label: &str, preview: &str, enter: Action, group: Option<&str>) -> PickerRow {
    PickerRow {
        label: label.to_string(),
        preview: Some(Preview::Text(preview.to_string())),
        enter,
        keys: Vec::new(),
        group: group.map(str::to_string),
    }
}

#[test]
fn the_pickers_source_lists_each_picker_by_key_and_name_and_enter_starts_it() {
    let rows = catalog_rows("pickers", &some_pickers(), CHORD_TABLE);

    assert_eq!(
        rows,
        [
            row(
                &format!("ctrl-t m       notes     {DIM}Notes in the vault{RESET}"),
                "Notes in the vault\n\nKey     ctrl-t m\nPicker  notes",
                start("notes"),
                None,
            ),
            row(
                &format!("ctrl-t ctrl-b  bookmark  {DIM}Bookmark this folder{RESET}"),
                "Bookmark this folder\n\nKey     ctrl-t ctrl-b\nPicker  bookmark",
                Action::Nothing,
                None,
            ),
            row(
                &format!("ctrl-t ;       pickers   {DIM}Every picker{RESET}"),
                "Every picker\n\nKey     ctrl-t ;\nPicker  pickers",
                Action::Nothing,
                None,
            ),
        ]
    );
}

#[test]
fn the_bindings_source_lists_herdr_pickers_keys_then_the_chord_tables_acting_by_kind() {
    let rows = catalog_rows("bindings", &some_pickers()[..1], CHORD_TABLE);

    assert_eq!(
        rows,
        [
            row(
                "ctrl-t m  Notes in the vault",
                "Notes in the vault\n\nKey     ctrl-t m\nKind    picker\nAction  notes",
                start("notes"),
                Some("herdr-picker"),
            ),
            row(
                "ctrl-a    beginning-of-line",
                "Key     ctrl-a\nKind    command\nAction  beginning-of-line",
                Action::Nothing,
                Some("editing"),
            ),
            row(
                "alt-l     ls/eza",
                "ls/eza\n\nKey     alt-l\nKind    run\nAction  eza -ahlrs date",
                Action::Run("eza -ahlrs date".to_string()),
                Some("listing"),
            ),
            row(
                "alt-L     eza -ahlrs date",
                "Key     alt-L\nKind    insert\nAction  eza -ahlrs date",
                Action::Insert("eza -ahlrs date".to_string()),
                Some("listing"),
            ),
            row(
                "ctrl-s    Insert 'sudo ' at the beginning of a command.",
                "Insert 'sudo ' at the beginning of a command.\n\nKey     ctrl-s\nKind    function\nAction  sudo_insert",
                Action::Insert("sudo_insert".to_string()),
                Some("sudo"),
            ),
        ]
    );
}

#[test]
fn without_a_chord_table_the_bindings_are_herdr_pickers_own() {
    let rows = catalog_rows("bindings", &some_pickers()[..1], "");

    assert_eq!(
        rows.iter()
            .map(|row| row.label.as_str())
            .collect::<Vec<_>>(),
        ["ctrl-t m  Notes in the vault"]
    );
}
