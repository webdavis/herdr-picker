use herdr_picker_domain::PickerCommand;

use super::*;

const NOTES: &str = r#"
[pickers.notes]
key = "ctrl-t m"
title = "Vault"
description = "Notes"
command = "notes.py"
sources = ["folders", "tags", "recent"]

[pickers.notes.steps.note-actions]
command = "notes.py actions"

[pickers.notes.steps.checkout]
command = "notes.py checkout"
"#;

fn source_of(step: &StepCommand) -> Option<String> {
    step.command
        .get_envs()
        .find(|(name, _)| *name == OsStr::new(PICKER_SOURCE_VARIABLE))
        .and_then(|(_, value)| value)
        .map(|value| value.to_string_lossy().into_owned())
}

fn script(config: &PluginConfig, words: &[&str]) -> Vec<String> {
    let PickerCommand::Program(program) = &config.pickers[0].command else {
        panic!("notes runs a script");
    };
    std::iter::once(program[0].clone())
        .chain(words.iter().map(|word| word.to_string()))
        .collect()
}

#[test]
fn a_scripted_picker_reads_its_first_source_until_another_is_chosen() {
    let config = config("scripted-sources", NOTES);

    let first = step_command(&call("notes", &[], None), None, &config, &[]).expect("a command");
    let chosen =
        step_command(&call("notes", &[], None), Some("tags"), &config, &[]).expect("a command");

    assert_eq!(first.sources, ["folders", "tags", "recent"]);
    assert_eq!(first.source.as_deref(), Some("folders"));
    assert_eq!(source_of(&first).as_deref(), Some("folders"));
    assert_eq!(source_of(&chosen).as_deref(), Some("tags"));
    assert!(!first.live);
}

#[test]
fn a_next_naming_a_step_of_a_picker_on_the_trail_runs_that_step_with_its_args() {
    let config = config("scripted-step", NOTES);

    let step = step_command(
        &call("note-actions", &["a.md"], Some("{}")),
        Some("tags"),
        &config,
        &["pickers", "notes", "note-actions"],
    )
    .expect("a command");

    assert_eq!(words(&step.command), script(&config, &["actions", "a.md"]));
    assert_eq!(step.sources, Vec::<String>::new());
    assert_eq!(source_of(&step), None);
}

#[test]
fn a_pickers_own_step_wins_over_a_builtin_step_of_the_same_name() {
    let config = config("scripted-step-shadow", NOTES);

    let own_step =
        step_command(&call("checkout", &[], None), None, &config, &["notes"]).expect("a command");
    let elsewhere =
        step_command(&call("checkout", &[], None), None, &config, &["files"]).expect("a command");

    assert_eq!(words(&own_step.command), script(&config, &["checkout"]));
    assert_eq!(words(&elsewhere.command)[1..], ["builtin", "checkout"]);
}

#[test]
fn a_step_off_the_trail_is_no_picker() {
    let config = config("scripted-step-off-trail", NOTES);

    let refusal =
        step_command(&call("note-actions", &[], None), None, &config, &["files"]).map(|_| ());

    assert!(
        matches!(&refusal, Err(CommandError::Failed(reason)) if reason == "no picker is named note-actions"),
        "{refusal:?}"
    );
}

#[test]
fn a_picker_is_titled_as_the_config_says_and_a_step_by_its_name() {
    let config = config("scripted-title", NOTES);

    assert_eq!(picker_title("notes", &config), "Vault");
    assert_eq!(picker_title("note-actions", &config), "Note Actions");
    assert_eq!(picker_title("files", &config), "Files");
}
