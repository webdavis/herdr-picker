use super::*;

fn source_of(step: &StepCommand) -> Option<String> {
    step.command
        .get_envs()
        .find(|(variable, _)| *variable == OsStr::new(PICKER_SOURCE_VARIABLE))
        .and_then(|(_, value)| value)
        .map(|value| value.to_string_lossy().into_owned())
}

#[test]
fn a_builtin_with_sources_reads_its_first_source_unless_another_was_chosen() {
    let config = config("sources", "");

    let first = step_command(&call("files", &[], None), None, &config, &[]).expect("a command");
    let chosen =
        step_command(&call("files", &[], None), Some("home"), &config, &[]).expect("a command");

    assert_eq!(source_of(&first), Some("this folder".to_string()));
    assert_eq!(first.source, Some("this folder".to_string()));
    assert_eq!(
        first.sources,
        ["this folder", "hidden", "home", "whole disk", "recent"]
    );
    assert_eq!(source_of(&chosen), Some("home".to_string()));
    assert_eq!(chosen.source, Some("home".to_string()));
}

#[test]
fn a_step_builtin_with_sources_reads_its_first_unless_another_was_chosen() {
    let config = config("step-sources", "");
    let file = call("data-file", &["/a.json"], None);

    let first = step_command(&file, None, &config, &[]).expect("a command");
    let chosen = step_command(&file, Some("levels"), &config, &[]).expect("a command");

    assert_eq!(source_of(&first), Some("paths".to_string()));
    assert_eq!(
        first.sources,
        ["paths", "zoom", "keys", "levels", "shape", "tables"]
    );
    assert_eq!(source_of(&chosen), Some("levels".to_string()));
    assert_eq!(chosen.source, Some("levels".to_string()));
}
