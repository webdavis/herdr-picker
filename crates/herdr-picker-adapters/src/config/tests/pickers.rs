use herdr_picker_domain::{Chord, Picker, PickerCommand, PickerError};

use super::*;

const TWO_PICKERS: &str = r#"
[pickers.files]
key = "ctrl-t f"
description = "Files in this repository"
command = "files.sh"

[pickers.branches]
key = "ctrl-g b"
description = "Git branches"
command = "branches.sh"
"#;

#[test]
fn a_config_of_only_pickers_is_valid_and_declares_no_workspace() {
    let sandbox = TempDirectory::new("only-pickers");
    let path = sandbox.write(TWO_PICKERS);

    let config = read_config(&path).expect("a readable config");

    assert_eq!(config.targets, Vec::new());
    assert_eq!(
        config.pickers[..2],
        [
            Picker {
                name: "branches".to_string(),
                chord: Chord::parse("ctrl-g b").expect("a chord"),
                description: "Git branches".to_string(),
                command: script(&sandbox, "branches.sh", &[]),
                args: Vec::new(),
            },
            Picker {
                name: "files".to_string(),
                chord: Chord::parse("ctrl-t f").expect("a chord"),
                description: "Files in this repository".to_string(),
                command: script(&sandbox, "files.sh", &[]),
                args: Vec::new(),
            },
        ]
    );
}

#[test]
fn pickers_sit_beside_workspaces() {
    let sandbox = TempDirectory::new("pickers-and-workspaces");
    let path = sandbox.write(&format!(
        "{TWO_PICKERS}{FIVE_WORKSPACES_DECLARED_OUT_OF_ORDER}"
    ));

    let config = read_config(&path).expect("a readable config");

    assert_eq!(config.targets.len(), 5);
    assert_eq!(
        names(&config.pickers),
        [
            "branches",
            "files",
            "folders",
            "bookmark",
            "status",
            "history",
            "contents",
            "git-config",
            "recipes",
            "tests"
        ]
    );
}

#[test]
fn a_config_without_pickers_runs_the_builtin_ones_on_their_keys() {
    let sandbox = TempDirectory::new("no-pickers");
    let path = sandbox.write(FIVE_WORKSPACES_DECLARED_OUT_OF_ORDER);

    assert_eq!(
        read_config(&path).expect("a readable config").pickers,
        vec![
            builtin(
                "files",
                "ctrl-t f",
                "Files here, hidden ones, at home, on the whole disk, or used lately"
            ),
            builtin("folders", "ctrl-t d", "Folders to change into"),
            builtin(
                "bookmark",
                "ctrl-t ctrl-b",
                "Bookmark this folder for the folders picker"
            ),
            builtin(
                "status",
                "ctrl-g s",
                "Every file Git has an opinion about, and the stashes"
            ),
            builtin(
                "history",
                "ctrl-g l",
                "Commits, the reflog, or the commits whose changes match what is typed"
            ),
            builtin(
                "branches",
                "ctrl-g b",
                "Branches, tags and worktrees, or the repos with something to prune"
            ),
            builtin("contents", "ctrl-g /", "Lines matching what is typed"),
            builtin(
                "git-config",
                "ctrl-g v",
                "Git settings, where each comes from, and which value wins"
            ),
            builtin("recipes", "ctrl-t j", "Recipes in this project's justfile"),
            builtin(
                "tests",
                "ctrl-t t",
                "Tests in this project, compiler and linter errors, or the saved quickfix list"
            ),
        ]
    );
}

#[test]
fn a_picker_with_an_unknown_field_is_refused() {
    let (refusal, rendered) = refusal_for(
        "picker-unknown-field",
        "[pickers.files]\nkey = \"ctrl-t f\"\ncommand = \"files.sh\"\nshell = \"zsh\"\n",
    );

    assert!(
        matches!(refusal, ConfigError::Unparseable { line: Some(4), .. }),
        "{rendered}"
    );
    assert!(rendered.contains("unknown field `shell`"), "{rendered}");
}

#[test]
fn two_pickers_on_the_same_key_are_refused_in_one_sentence_naming_both() {
    let (refusal, rendered) = refusal_for(
        "picker-key-collision",
        "[pickers.branches]\nkey = \"ctrl-t b\"\ncommand = \"branches.sh\"\n\n[pickers.buffers]\nkey = \"ctrl-t b\"\ncommand = \"buffers.sh\"\n",
    );

    assert_eq!(
        refusal,
        ConfigError::InvalidPicker {
            path: refusal_path(&refusal),
            cause: PickerError::CollidingKey {
                first: "branches".to_string(),
                second: "buffers".to_string(),
                key: "ctrl-t b".to_string(),
            },
        }
    );
    assert!(
        rendered.ends_with(": the pickers branches and buffers both run on ctrl-t b"),
        "{rendered}"
    );
}

fn script(sandbox: &TempDirectory, name: &str, arguments: &[&str]) -> PickerCommand {
    let program = sandbox.0.join("pickers").join(name);
    PickerCommand::Program(
        std::iter::once(program.to_string_lossy().into_owned())
            .chain(arguments.iter().map(|argument| argument.to_string()))
            .collect(),
    )
}

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
fn a_builtin_picker_names_one_of_herdr_pickers_own_and_may_declare_args() {
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

fn names(pickers: &[Picker]) -> Vec<&str> {
    pickers.iter().map(|picker| picker.name.as_str()).collect()
}

fn builtin(name: &str, key: &str, description: &str) -> Picker {
    Picker {
        name: name.to_string(),
        chord: Chord::parse(key).expect("a chord"),
        description: description.to_string(),
        command: PickerCommand::Builtin(name.to_string()),
        args: Vec::new(),
    }
}

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
            "tests"
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
            "tests"
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

#[test]
fn without_a_config_file_the_pickers_are_the_builtin_ones() {
    let sandbox = TempDirectory::new("picker-config-missing");

    let config = read_picker_config(&sandbox.0.join("config.toml")).expect("the defaults");

    assert_eq!(
        names(&config.pickers),
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
            "tests"
        ]
    );
    assert_eq!(config.targets, Vec::new());
}

fn refusal_path(refusal: &ConfigError) -> PathBuf {
    match refusal {
        ConfigError::InvalidPicker { path, .. } => path.clone(),
        other => panic!("expected a picker refusal, got {other:?}"),
    }
}
