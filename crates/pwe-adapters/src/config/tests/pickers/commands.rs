use super::*;

fn only_picker(sandbox: &TempDirectory, body: &str) -> Picker {
    let path = sandbox.write(&format!("[pickers.files]\nkey = \"ctrl-t f\"\n{body}"));
    let mut pickers = read_config(&path).expect("a readable config").pickers;
    assert_eq!(pickers[0].name, "files");
    pickers.remove(0)
}

#[test]
fn a_command_line_splits_into_the_program_and_its_arguments_like_the_shell() {
    let sandbox = TempDirectory::new("picker-command-line");

    let picker = only_picker(&sandbox, "command = \"files.sh --hidden 'my dir'\"\n");

    assert_eq!(
        picker.command,
        script(&sandbox, "files.sh", &["--hidden", "my dir"])
    );
}

#[test]
fn a_command_may_be_an_array_and_an_absolute_program_stays_where_it_is() {
    let sandbox = TempDirectory::new("picker-command-array");

    let picker = only_picker(&sandbox, "command = [\"/usr/bin/env\", \"files\"]\n");

    assert_eq!(
        picker.command,
        PickerCommand::Program(vec!["/usr/bin/env".to_string(), "files".to_string()])
    );
}

#[test]
fn a_builtin_picker_names_one_of_pwe_own_and_may_declare_args() {
    let sandbox = TempDirectory::new("picker-builtin");

    let picker = only_picker(
        &sandbox,
        "builtin = \"echo-args\"\nargs = [\"a\", \"b c\"]\n",
    );

    assert_eq!(
        picker.command,
        PickerCommand::Builtin("echo-args".to_string())
    );
    assert_eq!(picker.args, vec!["a".to_string(), "b c".to_string()]);
}

#[test]
fn a_picker_needs_exactly_one_of_command_or_builtin() {
    for (name, body) in [
        (
            "picker-both",
            "command = \"files.sh\"\nbuiltin = \"echo-args\"\n",
        ),
        ("picker-neither", "description = \"none\"\n"),
    ] {
        let (refusal, rendered) = refusal_for(
            name,
            &format!("[pickers.notes]\nkey = \"ctrl-t n\"\n{body}"),
        );

        assert_eq!(
            refusal,
            ConfigError::InvalidPicker {
                path: refusal_path(&refusal),
                cause: PickerError::UnclearCommand {
                    name: "notes".to_string()
                },
            }
        );
        assert!(
            rendered.ends_with(": the picker notes needs exactly one of command or builtin"),
            "{rendered}"
        );
    }
}

#[test]
fn a_command_line_with_an_unclosed_quote_or_no_program_is_refused() {
    for (name, command) in [("picker-quote", "files.sh 'oops"), ("picker-empty", "  ")] {
        let (refusal, rendered) = refusal_for(
            name,
            &format!("[pickers.files]\nkey = \"ctrl-t f\"\ncommand = \"{command}\"\n"),
        );

        assert!(
            matches!(refusal, ConfigError::InvalidPicker { .. }),
            "{rendered}"
        );
        assert!(
            rendered.ends_with(&format!(
                ": the picker files runs {command:?}, which is not a program then its arguments"
            )),
            "{rendered}"
        );
    }
}
