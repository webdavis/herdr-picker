use herdr_picker_domain::PickerStep;

use super::*;

const NOTES: &str = r#"
[pickers.notes]
key = "ctrl-t m"
title = "Vault"
description = "Notes by folder, tag, or lately opened"
command = "notes.py --vault ~/notes"
sources = ["folders", "tags", "recent"]

live = true

[pickers.notes.steps.note-actions]
command = "notes.py actions"
sources = ["open", "move"]
live = true

[pickers.notes.steps.links]
command = ["/opt/links", "--all"]
"#;

#[test]
fn a_scripted_picker_declares_its_title_sources_steps_and_live() {
    let sandbox = TempDirectory::new("picker-sources");
    let pickers = read_config(&sandbox.write(NOTES))
        .expect("a readable config")
        .pickers;

    let notes = &pickers[0];
    assert_eq!(notes.name, "notes");
    assert_eq!(notes.title, "Vault");
    assert_eq!(notes.sources, ["folders", "tags", "recent"]);
    assert!(notes.live);
    assert_eq!(
        notes.steps,
        BTreeMap::from([
            (
                "links".to_string(),
                PickerStep {
                    command: vec!["/opt/links".to_string(), "--all".to_string()],
                    sources: Vec::new(),
                    live: false,
                }
            ),
            (
                "note-actions".to_string(),
                PickerStep {
                    command: match script(&sandbox, "notes.py", &["actions"]) {
                        PickerCommand::Program(words) => words,
                        PickerCommand::Builtin(_) => unreachable!(),
                    },
                    sources: vec!["open".to_string(), "move".to_string()],
                    live: true,
                }
            ),
        ])
    );
}

#[test]
fn a_builtin_picker_set_live_is_refused() {
    let (_, rendered) = refusal_for("builtin-live", "[pickers.files]\nlive = true\n");

    assert!(
        rendered.ends_with(
            ": the picker files runs a built-in picker, which decides itself whether it follows the query"
        ),
        "{rendered}"
    );
}

#[test]
fn a_step_naming_anything_but_a_command_is_refused_at_its_line() {
    let (refusal, rendered) = refusal_for(
        "step-unknown-field",
        "[pickers.notes]\nkey = \"ctrl-t m\"\ncommand = \"notes.py\"\n[pickers.notes.steps.open]\nbuiltin = \"files\"\n",
    );

    assert!(
        matches!(refusal, ConfigError::Unparseable { line: Some(5), .. }),
        "{rendered}"
    );
    assert!(rendered.contains("unknown field `builtin`"), "{rendered}");
}

#[test]
fn a_step_whose_command_names_no_program_is_refused_naming_the_step() {
    let (_, rendered) = refusal_for(
        "step-empty-command",
        "[pickers.notes]\nkey = \"ctrl-t m\"\ncommand = \"notes.py\"\n[pickers.notes.steps.open]\ncommand = \"\"\n",
    );

    assert!(
        rendered.ends_with(
            ": the picker notes.steps.open runs \"\", which is not a program then its arguments"
        ),
        "{rendered}"
    );
}

#[test]
fn a_builtin_picker_given_sources_is_refused() {
    let (_, rendered) = refusal_for("builtin-sources", "[pickers.files]\nsources = [\"mine\"]\n");

    assert!(
        rendered
            .ends_with(": the picker files runs a built-in picker, which reads its own sources"),
        "{rendered}"
    );
}
