use super::*;

fn declared(name: &str, key: &str) -> PickerDeclaration {
    PickerDeclaration {
        name: name.to_string(),
        key: key.to_string(),
        description: format!("{name} picker"),
        command: PickerCommand::Program(vec![format!("/config/pickers/{name}.sh")]),
        args: Vec::new(),
    }
}

#[test]
fn pickers_parse_each_key_into_a_chord_and_keep_the_rest() {
    let accepted = pickers([
        declared("branches", "ctrl-g b"),
        declared("files", "ctrl-t f"),
    ]);

    assert_eq!(
        accepted,
        Ok(vec![
            Picker {
                name: "branches".to_string(),
                chord: Chord::parse("ctrl-g b").expect("a chord"),
                description: "branches picker".to_string(),
                command: PickerCommand::Program(vec!["/config/pickers/branches.sh".to_string()]),
                args: Vec::new(),
            },
            Picker {
                name: "files".to_string(),
                chord: Chord::parse("ctrl-t f").expect("a chord"),
                description: "files picker".to_string(),
                command: PickerCommand::Program(vec!["/config/pickers/files.sh".to_string()]),
                args: Vec::new(),
            },
        ])
    );
}

#[test]
fn two_pickers_on_the_same_chord_are_refused_naming_both() {
    let refusal = pickers([
        declared("branches", "ctrl-t b"),
        declared("buffers", "ctrl-T  b"),
    ])
    .expect_err("a shared chord is refused");

    assert_eq!(
        refusal.to_string(),
        "the pickers branches and buffers both run on ctrl-T  b"
    );
}

#[test]
fn a_key_that_is_not_a_chord_is_refused_naming_the_picker() {
    let refusal = pickers([declared("files", "b")]).expect_err("a plain key is refused");

    assert_eq!(
        refusal,
        PickerError::UnusableKey {
            name: "files".to_string(),
            key: "b".to_string()
        }
    );
    assert_eq!(
        refusal.to_string(),
        "the picker files declares the key \"b\", which is not a chord: ctrl-<letter> or alt-<letter>, then any plain letters, like \"ctrl-t b\""
    );
}

#[test]
fn a_name_that_a_shell_key_binding_cannot_carry_bare_is_refused() {
    let refusal =
        pickers([declared("my files", "ctrl-t f")]).expect_err("a spaced name is refused");

    assert_eq!(
        refusal.to_string(),
        "the picker name \"my files\" is not only letters, digits, - and _"
    );
    assert!(pickers([declared("", "ctrl-t f")]).is_err());
}
