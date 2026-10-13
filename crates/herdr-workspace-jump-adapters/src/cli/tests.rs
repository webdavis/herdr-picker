use super::*;
use crate::cli_command::RecordedCli;
use crate::cli_command::spawning;

fn a_herdr_that_died_before_answering() -> CliWorkspaceDirectory {
    CliWorkspaceDirectory::using(Some("false".to_string()))
}

fn a_herdr_that_answers_with_its_arguments() -> CliWorkspaceDirectory {
    CliWorkspaceDirectory::using(Some("echo".to_string()))
}

fn a_herdr_that_acknowledges_silently() -> CliWorkspaceDirectory {
    CliWorkspaceDirectory::using(Some("true".to_string()))
}

#[test]
fn using_prefers_the_injected_path_and_falls_back_to_the_bare_name() {
    let _spawning = spawning();
    assert_eq!(
        CliWorkspaceDirectory::using(Some("/opt/herdr".to_string())).binary,
        "/opt/herdr"
    );
    assert_eq!(CliWorkspaceDirectory::using(None).binary, "herdr");
    assert_eq!(
        CliWorkspaceDirectory::using(Some(String::new())).binary,
        "herdr"
    );
}

#[test]
fn a_binary_that_cannot_be_spawned_is_unreachable() {
    let _spawning = spawning();
    let mut directory = CliWorkspaceDirectory::using(Some("/nonexistent/herdr-binary".to_string()));

    assert!(matches!(directory.list(), Err(JumpError::Unreachable(_))));
}

#[test]
fn a_non_zero_exit_with_no_envelope_is_a_transport_failure() {
    let _spawning = spawning();
    let mut directory = a_herdr_that_died_before_answering();

    assert!(matches!(directory.list(), Err(JumpError::Transport(_))));
}

#[test]
fn an_error_envelope_is_reported_even_when_the_command_exits_zero() {
    let _spawning = spawning();
    let failure = a_herdr_that_answers_with_its_arguments()
        .run(&[r#"{"error":{"code":"not_found","message":"gone"}}"#]);

    assert_eq!(
        failure,
        Err(JumpError::Server {
            code: "not_found".to_string(),
            message: "gone".to_string(),
        })
    );
}

#[test]
fn a_silent_successful_cli_acknowledgment_is_accepted() {
    let _spawning = spawning();
    assert_eq!(
        a_herdr_that_acknowledges_silently().run(&["workspace", "focus", "wA"]),
        Ok(json!({}))
    );
}

#[test]
fn a_successful_envelope_is_parsed_into_workspaces() {
    let _spawning = spawning();
    let replayed = a_herdr_that_answers_with_its_arguments()
        .run(&[r#"{"result":{"workspaces":[{"workspace_id":"wA","label":"netpulse"}]}}"#]);

    let result = match replayed {
        Ok(result) => result,
        Err(error) => panic!("expected the envelope to parse: {error}"),
    };
    assert_eq!(
        response::workspaces(&result),
        Ok(vec![Workspace {
            workspace_id: "wA".to_string(),
            label: "netpulse".to_string(),
        }])
    );
}

#[test]
fn list_executes_the_actual_cli_and_maps_its_workspace_records() {
    let _spawning = spawning();
    let fake = RecordedCli::answering(
        r#"{"result":{"workspaces":[{"workspace_id":"wA","label":"Ivy vault "}]}}"#,
        "",
    );
    let mut directory = CliWorkspaceDirectory::using(Some(fake.binary.clone()));

    assert_eq!(
        directory.list(),
        Ok(vec![Workspace {
            workspace_id: "wA".into(),
            label: "Ivy vault ".into()
        }])
    );
    assert_eq!(fake.calls(), vec![vec!["workspace", "list"]]);
}

#[test]
fn focus_executes_the_actual_cli_with_the_exact_workspace_id() {
    let _spawning = spawning();
    let fake = RecordedCli::answering("", "");
    let mut directory = CliWorkspaceDirectory::using(Some(fake.binary.clone()));

    assert_eq!(directory.focus("wA"), Ok(()));
    assert_eq!(fake.calls(), vec![vec!["workspace", "focus", "wA"]]);
}

#[test]
fn create_executes_the_actual_cli_with_distinct_cwd_label_and_focus_arguments() {
    let _spawning = spawning();
    let fake = RecordedCli::answering("", "");
    let mut directory = CliWorkspaceDirectory::using(Some(fake.binary.clone()));

    assert_eq!(
        directory.create("Ivy vault's label ", "/tmp/a project's root"),
        Ok(())
    );
    assert_eq!(
        fake.calls(),
        vec![vec![
            "workspace",
            "create",
            "--cwd",
            "/tmp/a project's root",
            "--label",
            "Ivy vault's label ",
            "--focus"
        ]]
    );
}
