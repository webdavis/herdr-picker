use super::*;

fn pickers_of(name: &str, content: &str) -> Vec<Picker> {
    let sandbox = TempDirectory::new(name);
    read_config(&sandbox.write(content))
        .expect("a readable config")
        .pickers
}

#[test]
fn a_table_named_for_a_builtin_picker_moves_its_key_or_turns_it_off() {
    let pickers = pickers_of(
        "builtin-override",
        "[pickers.files]\nkey = \"alt-f\"\n\n[pickers.bookmark]\nenabled = false\n",
    );

    assert_eq!(
        pickers[..2],
        [
            builtin(
                "files",
                "alt-f",
                "Files here, hidden ones, at home, on the whole disk, or used lately"
            ),
            builtin("folders", "ctrl-t d", "Folders to change into"),
        ]
    );
    assert!(!names(&pickers).contains(&"bookmark"));
}

#[test]
fn a_builtin_picker_gives_its_key_to_a_configured_picker_that_claims_it() {
    let pickers = pickers_of(
        "builtin-yields",
        "[pickers.dirs]\nkey = \"ctrl-t d\"\ncommand = \"dirs.sh\"\n",
    );

    assert_eq!(
        names(&pickers),
        [
            "dirs",
            "files",
            "bookmark",
            "status",
            "history",
            "branches",
            "contents",
            "git-config",
            "recipes",
            "tests",
            "processes",
            "network",
            "data",
            "agents",
            "pickers",
            "apple",
            "tabs"
        ]
    );
}

#[test]
fn a_configured_picker_turned_off_is_not_declared() {
    let pickers = pickers_of(
        "user-off",
        "[pickers.notes]\nkey = \"ctrl-t n\"\ncommand = \"notes.sh\"\nenabled = false\n",
    );

    assert_eq!(
        names(&pickers),
        [
            "files",
            "folders",
            "bookmark",
            "status",
            "history",
            "branches",
            "contents",
            "git-config",
            "recipes",
            "tests",
            "processes",
            "network",
            "data",
            "agents",
            "pickers",
            "apple",
            "tabs"
        ]
    );
}

#[test]
fn a_builtin_moved_onto_a_configured_pickers_key_is_refused() {
    let (refusal, rendered) = refusal_for(
        "builtin-collides",
        "[pickers.files]\nkey = \"ctrl-t n\"\n\n[pickers.notes]\nkey = \"ctrl-t n\"\ncommand = \"notes.sh\"\n",
    );

    assert!(
        matches!(
            refusal,
            ConfigError::InvalidPicker {
                cause: PickerError::CollidingKey { .. },
                ..
            }
        ),
        "{rendered}"
    );
}

#[test]
fn a_configured_picker_without_a_key_is_refused() {
    let (_, rendered) = refusal_for("no-key", "[pickers.notes]\ncommand = \"notes.sh\"\n");

    assert!(
        rendered.ends_with(": the picker notes declares the key \"\", which is not a chord: ctrl-<letter> or alt-<letter>, then any plain letters or punctuation, like \"ctrl-t b\""),
        "{rendered}"
    );
}
