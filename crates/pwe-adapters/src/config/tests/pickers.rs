use std::collections::BTreeMap;

use pwe_domain::{BUILTIN_PICKERS, Chord, Picker, PickerCommand, PickerError};

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
                title: "Branches".to_string(),
                chord: Chord::parse("ctrl-g b").expect("a chord"),
                description: "Git branches".to_string(),
                command: script(&sandbox, "branches.sh", &[]),
                args: Vec::new(),
                sources: Vec::new(),
                steps: BTreeMap::new(),
                live: false,
                engine: pwe_domain::Engine::Fzf,
            },
            Picker {
                name: "files".to_string(),
                title: "Files".to_string(),
                chord: Chord::parse("ctrl-t f").expect("a chord"),
                description: "Files in this repository".to_string(),
                command: script(&sandbox, "files.sh", &[]),
                args: Vec::new(),
                sources: Vec::new(),
                steps: BTreeMap::new(),
                live: false,
                engine: pwe_domain::Engine::Fzf,
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

fn names(pickers: &[Picker]) -> Vec<&str> {
    pickers.iter().map(|picker| picker.name.as_str()).collect()
}

fn builtin(name: &str, key: &str, description: &str) -> Picker {
    Picker {
        name: name.to_string(),
        title: pwe_domain::title(name),
        chord: Chord::parse(key).expect("a chord"),
        description: description.to_string(),
        command: PickerCommand::Builtin(name.to_string()),
        args: Vec::new(),
        sources: BUILTIN_PICKERS
            .iter()
            .find(|builtin| builtin.name == name)
            .map_or(Vec::new(), |builtin| {
                builtin
                    .sources
                    .iter()
                    .map(|source| source.to_string())
                    .collect()
            }),
        steps: BTreeMap::new(),
        live: false,
        engine: pwe_domain::Engine::Fzf,
    }
}

fn refusal_path(refusal: &ConfigError) -> PathBuf {
    match refusal {
        ConfigError::InvalidPicker { path, .. } => path.clone(),
        other => panic!("expected a picker refusal, got {other:?}"),
    }
}

mod bluebubbles;
mod builtins;
mod commands;
mod engines;
mod overrides;
mod sources;
mod unifi;
