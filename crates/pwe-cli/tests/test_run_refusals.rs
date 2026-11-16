mod test_project;

use serde_json::{Value, json};
use test_project::{TestProject, exit_code, json_of};

const MISSING_TOOL_RUNNER: &str = r#"name = "absent"
language = "none"
status = "untested"
when = "The folder holds absent.txt."
does = "Runs a program no machine has."

[detect]
files = ["absent.txt"]

[run]
all = ["no-such-test-tool-anywhere"]

[output]
parser = "libtest"
report = "stdout"
"#;

fn refusal(project: &TestProject, words: &[&str]) -> Value {
    let output = project.pwe(words);
    assert_eq!(exit_code(&output), Some(2), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    json_of(&String::from_utf8_lossy(&output.stderr))["error"].clone()
}

#[test]
fn a_folder_no_runner_claims_is_refused_with_no_runner() {
    let project = TestProject::new("refuse-no-runner");

    let error = refusal(&project, &["test", "run", "--all", "--format", "jsonl"]);

    assert_eq!(error["code"], json!("NO_RUNNER"));
    assert_eq!(error["hint"]["command"], json!("pwe test runners"));
}

#[test]
fn a_run_without_a_target_is_refused_with_the_accepted_targets() {
    let project = TestProject::new("refuse-no-target");
    project.file("Cargo.toml", "");

    let error = refusal(&project, &["test", "run", "--format", "jsonl"]);

    assert_eq!(
        (&error["code"], &error["hint"]["accepted"]),
        (&json!("BAD_TARGET"), &json!(["--all", "--file", "--id"]))
    );
}

#[test]
fn a_file_target_the_cargo_runner_cannot_run_is_refused() {
    let project = TestProject::new("refuse-file");
    project.file("Cargo.toml", "");

    let error = refusal(
        &project,
        &["test", "run", "--file", "src/lib.rs", "--format", "json"],
    );

    assert_eq!(error["code"], json!("BAD_TARGET"));
}

#[test]
fn an_id_that_is_not_a_test_id_is_refused() {
    let project = TestProject::new("refuse-bad-id");
    project.file("Cargo.toml", "");

    let error = refusal(
        &project,
        &["test", "run", "--id", "tests::passes", "--format", "json"],
    );

    assert_eq!(error["code"], json!("BAD_TARGET"));
}

#[test]
fn a_tool_missing_from_the_path_is_refused_with_no_event() {
    let project = TestProject::new("refuse-missing-tool");
    project
        .file("absent.txt", "")
        .user_runner("absent.toml", MISSING_TOOL_RUNNER);

    let error = refusal(&project, &["test", "run", "--all", "--format", "jsonl"]);

    assert_eq!(error["code"], json!("TOOL_MISSING"));
}

#[test]
fn a_runner_named_with_runner_that_does_not_match_lists_the_ones_that_do() {
    let project = TestProject::new("refuse-runner");
    project.file("Cargo.toml", "");

    let error = refusal(
        &project,
        &[
            "test", "run", "--all", "--runner", "pytest", "--format", "json",
        ],
    );

    assert_eq!(
        (&error["code"], &error["hint"]["available"]),
        (&json!("NO_RUNNER"), &json!(["cargo"]))
    );
}
