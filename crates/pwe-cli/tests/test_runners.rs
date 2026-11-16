mod test_project;

use serde_json::json;
use test_project::{TestProject, exit_code, json_of, stdout_of};

const BROKEN_RUNNER: &str = "name = \"broken\"\n";

fn runners(project: &TestProject) -> serde_json::Value {
    let output = project.pwe(&["test", "runners", "--format", "json"]);
    assert_eq!(exit_code(&output), Some(0), "{output:?}");
    json_of(&stdout_of(&output))
}

#[test]
fn runners_names_cargo_and_the_file_that_matched() {
    let project = TestProject::new("runners-cargo");
    project.file("Cargo.toml", "[workspace]\n");

    assert_eq!(
        runners(&project)["runners"],
        json!([{
            "name": "cargo",
            "language": "rust",
            "status": "tested",
            "matched-by": "Cargo.toml",
            "command-prefix": [],
        }])
    );
}

#[test]
fn a_folder_no_runner_claims_lists_none_and_exits_0() {
    let project = TestProject::new("runners-none");

    assert_eq!(runners(&project)["runners"], json!([]));
}

#[test]
fn a_mise_project_runs_every_command_through_mise_exec() {
    let project = TestProject::new("runners-mise");
    project
        .file("Cargo.toml", "[workspace]\n")
        .file("mise.toml", "");

    assert_eq!(
        runners(&project)["runners"][0]["command-prefix"],
        json!(["mise", "exec", "--"])
    );
}

#[test]
fn a_user_runner_file_of_the_same_name_wins_over_the_shipped_one() {
    let project = TestProject::new("runners-user");
    project.file("Cargo.toml", "[workspace]\n").user_runner(
        "cargo.toml",
        &include_str!("../../pwe-adapters/runners/cargo.toml")
            .replace("language = \"rust\"", "language = \"rust-by-the-user\""),
    );

    assert_eq!(
        runners(&project)["runners"][0]["language"],
        json!("rust-by-the-user")
    );
}

#[test]
fn a_broken_runner_file_is_named_while_the_others_still_work() {
    let project = TestProject::new("runners-broken");
    project
        .file("Cargo.toml", "[workspace]\n")
        .user_runner("broken.toml", BROKEN_RUNNER);
    let listed = runners(&project);

    assert_eq!(listed["runners"][0]["name"], json!("cargo"));
    assert_eq!(listed["broken-runner-files"][0]["field"], json!("language"));
}

#[test]
fn runners_in_text_says_which_file_matched() {
    let project = TestProject::new("runners-text");
    project.file("Cargo.toml", "[workspace]\n");

    let output = project.pwe(&["test", "runners", "--format", "text"]);

    assert!(stdout_of(&output).contains("cargo"), "{output:?}");
    assert!(stdout_of(&output).contains("Cargo.toml"), "{output:?}");
}

#[test]
fn an_unknown_flag_is_one_error_object_with_the_accepted_flags_and_exit_2() {
    let project = TestProject::new("runners-bad-flag");

    let output = project.pwe(&["test", "runners", "--fromat", "json"]);

    assert_eq!(exit_code(&output), Some(2));
    assert_eq!(
        json_of(&String::from_utf8_lossy(&output.stderr)),
        json!({"error": {
            "code": "BAD_FLAG",
            "message": "pwe test does not know the flag `--fromat`.",
            "hint": {"accepted": ["--format"]},
        }})
    );
}
