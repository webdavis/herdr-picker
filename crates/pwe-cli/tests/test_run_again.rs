mod test_project;

use serde_json::{Value, json};
use test_project::{TestProject, exit_code, json_of, lines_of};

const TWO_CRATES: &str =
    include_str!("../../pwe-protocol/src/test_engine/libtest/fixtures/cargo-test-two-crates.txt");
const PASSING: &str = "     Running unittests src/lib.rs (target/debug/deps/probe-1)\n\nrunning 1 test\ntest tests::passes ... ok\n";
const CARGO_TEST_IDS: [&str; 6] = [
    "test",
    "--no-fail-fast",
    "--message-format",
    "json",
    "--",
    "--exact",
];

fn cargo_project(name: &str) -> TestProject {
    let project = TestProject::new(name);
    project.file("Cargo.toml", "[workspace]\n");
    project
}

fn run(project: &TestProject, output: &str, exit: i32, target: &[&str]) -> Vec<Value> {
    project.fake_tool("cargo", output, exit);
    let words = [&["test", "run"], target, &["--format", "jsonl"]].concat();
    lines_of(&project.pwe(&words))
}

fn refusal(project: &TestProject, target: &str) -> (Option<i32>, Value) {
    let output = project.pwe(&["test", "run", target, "--format", "json"]);
    let error = json_of(&String::from_utf8_lossy(&output.stderr))["error"].clone();
    (exit_code(&output), error["code"].clone())
}

#[test]
fn run_failed_runs_only_the_tests_that_failed_in_the_last_run() {
    let project = cargo_project("again-failed");
    run(&project, TWO_CRATES, 101, &["--all"]);

    let events = run(&project, PASSING, 0, &["--failed"]);

    let mut expected = CARGO_TEST_IDS.to_vec();
    expected.push("tests::same_name");
    assert_eq!(project.tool_arguments("cargo"), expected);
    assert_eq!(
        events[0]["target"],
        json!({"kind": "failed", "value": null})
    );
}

#[test]
fn run_failed_after_a_green_run_is_refused_with_no_tests() {
    let project = cargo_project("again-green");
    run(&project, PASSING, 0, &["--all"]);

    assert_eq!(refusal(&project, "--failed"), (Some(2), json!("NO_TESTS")));
}

#[test]
fn run_failed_with_no_stored_run_is_refused_with_no_tests() {
    let project = cargo_project("again-nothing");

    assert_eq!(refusal(&project, "--failed"), (Some(2), json!("NO_TESTS")));
}

#[test]
fn run_last_repeats_the_previous_runs_target() {
    let project = cargo_project("again-last");
    let id = "cargo:src/lib.rs::tests::passes";
    run(&project, PASSING, 0, &["--id", id]);

    let events = run(&project, PASSING, 0, &["--last"]);

    let mut expected = CARGO_TEST_IDS.to_vec();
    expected.push("tests::passes");
    assert_eq!(project.tool_arguments("cargo"), expected);
    assert_eq!(events[0]["target"], json!({"kind": "ids", "value": [id]}));
}

#[test]
fn run_last_after_run_failed_reruns_what_failed_in_the_run_before() {
    let project = cargo_project("again-last-failed");
    run(&project, TWO_CRATES, 101, &["--all"]);
    run(&project, TWO_CRATES, 101, &["--failed"]);

    let events = run(&project, TWO_CRATES, 101, &["--last"]);

    assert_eq!(
        events[0]["target"],
        json!({"kind": "failed", "value": null})
    );
    assert_eq!(
        project.tool_arguments("cargo").last().map(String::as_str),
        Some("tests::same_name")
    );
}

#[test]
fn run_last_with_no_stored_run_is_refused_with_no_tests() {
    let project = cargo_project("again-last-nothing");

    assert_eq!(refusal(&project, "--last"), (Some(2), json!("NO_TESTS")));
}
