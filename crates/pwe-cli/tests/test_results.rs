mod test_project;

use serde_json::{Value, json};
use test_project::{TestProject, exit_code, json_of, lines_of, stdout_of};

const TWO_CRATES: &str =
    include_str!("../../pwe-protocol/src/test_engine/libtest/fixtures/cargo-test-two-crates.txt");
const PASSING: &str = "     Running unittests src/lib.rs (target/debug/deps/probe-1)\n\nrunning 1 test\ntest tests::passes ... ok\n";

fn cargo_project(name: &str) -> TestProject {
    let project = TestProject::new(name);
    project.file("Cargo.toml", "[workspace]\n");
    project
}

fn run_with(project: &TestProject, output: &str, exit: i32) -> String {
    project.fake_tool("cargo", output, exit);
    stdout_of(&project.pwe(&["test", "run", "--all", "--format", "jsonl"]))
}

fn run_id_of(streamed: &str) -> String {
    json_of(streamed.lines().next().unwrap_or_default())["run-id"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

#[test]
fn results_last_replays_the_stored_events_and_exits_with_the_runs_own_code() {
    let project = cargo_project("results-last");
    let streamed = run_with(&project, TWO_CRATES, 101);

    let output = project.pwe(&["test", "results", "--last", "--format", "jsonl"]);

    assert_eq!(stdout_of(&output), streamed);
    assert_eq!(exit_code(&output), Some(1));
}

#[test]
fn results_with_no_target_replays_the_last_run() {
    let project = cargo_project("results-default");
    run_with(&project, TWO_CRATES, 101);
    let streamed = run_with(&project, PASSING, 0);

    let output = project.pwe(&["test", "results", "--format", "jsonl"]);

    assert_eq!(
        (stdout_of(&output), exit_code(&output)),
        (streamed, Some(0))
    );
}

#[test]
fn results_run_replays_the_named_run() {
    let project = cargo_project("results-run");
    let failed = run_with(&project, TWO_CRATES, 101);
    run_with(&project, PASSING, 0);

    let output = project.pwe(&[
        "test",
        "results",
        "--run",
        &run_id_of(&failed),
        "--format",
        "jsonl",
    ]);

    assert_eq!((stdout_of(&output), exit_code(&output)), (failed, Some(1)));
}

#[test]
fn results_as_text_show_the_stored_run_as_a_run_shows_it() {
    let project = cargo_project("results-text");
    run_with(&project, TWO_CRATES, 101);
    let shown = stdout_of(&project.pwe(&["test", "run", "--all", "--format", "text"]));

    let replayed = stdout_of(&project.pwe(&["test", "results", "--last", "--format", "text"]));

    assert_eq!(replayed, shown);
}

#[test]
fn results_as_json_hold_every_stored_event_in_one_document() {
    let project = cargo_project("results-json");
    let streamed = run_with(&project, TWO_CRATES, 101);

    let document = json_of(&stdout_of(
        &project.pwe(&["test", "results", "--format", "json"]),
    ));

    let stored: Vec<Value> = streamed.lines().map(json_of).collect();
    assert_eq!(document, json!({ "events": stored }));
}

#[test]
fn an_unknown_run_is_refused_with_bad_target() {
    let project = cargo_project("results-unknown");
    run_with(&project, PASSING, 0);

    let output = project.pwe(&[
        "test",
        "results",
        "--run",
        "20200101T000000Z-0000",
        "--format",
        "json",
    ]);
    let error = json_of(&String::from_utf8_lossy(&output.stderr))["error"].clone();

    assert_eq!(exit_code(&output), Some(2));
    assert_eq!(error["code"], json!("BAD_TARGET"));
    assert!(lines_of(&output).is_empty());
}

#[test]
fn results_in_a_folder_with_no_stored_run_is_refused_with_bad_target() {
    let project = cargo_project("results-none");

    let output = project.pwe(&["test", "results", "--format", "json"]);
    let error = json_of(&String::from_utf8_lossy(&output.stderr))["error"].clone();

    assert_eq!(
        (
            exit_code(&output),
            &error["code"],
            &error["hint"]["command"]
        ),
        (Some(2), &json!("BAD_TARGET"), &json!("pwe test run --all"))
    );
}
