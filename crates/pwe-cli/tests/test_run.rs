mod test_project;

use serde_json::{Value, json};
use test_project::{TestProject, exit_code, json_of, lines_of, stdout_of};

const TWO_CRATES: &str =
    include_str!("../../pwe-protocol/src/test_engine/libtest/fixtures/cargo-test-two-crates.txt");
const PASSING: &str = "     Running unittests src/lib.rs (target/debug/deps/probe-1)\n\nrunning 1 test\ntest tests::passes ... ok\n";
const CARGO_TEST: [&str; 5] = [
    "cargo",
    "test",
    "--no-fail-fast",
    "--message-format",
    "json",
];

fn cargo_project(name: &str, output: &str, exit: i32) -> TestProject {
    let project = TestProject::new(name);
    project
        .file("Cargo.toml", "[workspace]\n")
        .fake_tool("cargo", output, exit);
    project
}

fn status_of(events: &[Value], id: &str) -> Value {
    events
        .iter()
        .find(|event| event["event"] == "test-finished" && event["id"] == id)
        .map_or(Value::Null, |event| event["status"].clone())
}

#[test]
fn run_all_ends_with_run_finished_failed_and_exits_1() {
    let project = cargo_project("run-all", TWO_CRATES, 101);

    let output = project.pwe(&["test", "run", "--all", "--format", "jsonl"]);
    let events = lines_of(&output);
    let last = events.last().cloned().unwrap_or_default();

    assert_eq!(exit_code(&output), Some(1));
    assert_eq!(
        (
            &last["event"],
            &last["status"],
            &last["exit-code"],
            &last["tool-exit-code"]
        ),
        (
            &json!("run-finished"),
            &json!("failed"),
            &json!(1),
            &json!(101)
        )
    );
    assert_eq!(
        last["counts"],
        json!({"passed": 4, "failed": 1, "skipped": 0, "errored": 0})
    );
}

#[test]
fn the_first_event_is_run_started_with_the_format_version_and_the_command() {
    let project = cargo_project("run-started", PASSING, 0);

    let events = lines_of(&project.pwe(&["test", "run", "--all", "--format", "jsonl"]));

    assert_eq!(
        (&events[0]["event"], &events[0]["v"], &events[0]["argv"]),
        (&json!("run-started"), &json!(1), &json!(CARGO_TEST))
    );
    assert_eq!(events[0]["target"], json!({"kind": "all", "value": null}));
}

#[test]
fn tests_of_two_crates_are_named_from_the_repo_root() {
    let project = cargo_project("run-ids", TWO_CRATES, 101);

    let events = lines_of(&project.pwe(&["test", "run", "--all", "--format", "jsonl"]));

    assert_eq!(
        (
            status_of(&events, "cargo:crates/alpha/src/lib.rs::tests::same_name"),
            status_of(&events, "cargo:crates/beta/src/lib.rs::tests::same_name"),
        ),
        (json!("passed"), json!("failed"))
    );
}

#[test]
fn a_passing_run_exits_0() {
    let project = cargo_project("run-passing", PASSING, 0);

    let output = project.pwe(&["test", "run", "--all", "--format", "jsonl"]);

    assert_eq!(exit_code(&output), Some(0), "{output:?}");
    assert_eq!(lines_of(&output).last().unwrap()["status"], json!("passed"));
}

#[test]
fn run_ids_passes_each_test_path_to_cargo_as_an_exact_filter() {
    let project = cargo_project("run-id", PASSING, 0);

    project.pwe(&[
        "test",
        "run",
        "--id",
        "cargo:src/lib.rs::tests::passes",
        "cargo:tests/it.rs::outside",
    ]);

    let mut expected: Vec<&str> = CARGO_TEST.to_vec();
    expected.extend(["--", "--exact", "tests::passes", "outside"]);
    assert_eq!(project.tool_arguments("cargo"), expected[1..]);
}

#[test]
fn a_tool_killed_mid_run_still_ends_with_run_finished() {
    let project = TestProject::new("run-killed");
    project
        .file("Cargo.toml", "[workspace]\n")
        .fake_tool_ending("cargo", PASSING, "kill -9 $$");

    let output = project.pwe(&["test", "run", "--all", "--format", "jsonl"]);
    let last = lines_of(&output).last().cloned().unwrap_or_default();

    assert_eq!(
        (&last["event"], &last["status"], last.get("tool-exit-code")),
        (&json!("run-finished"), &json!("errored"), None)
    );
    assert_eq!(exit_code(&output), Some(1));
}

#[test]
fn json_prints_one_document_whose_last_event_is_run_finished() {
    let project = cargo_project("run-json", TWO_CRATES, 101);

    let output = project.pwe(&["test", "run", "--all", "--format", "json"]);
    let document = json_of(&stdout_of(&output));
    let events = document["events"].as_array().cloned().unwrap_or_default();

    assert_eq!(
        events.last().map(|last| &last["event"]),
        Some(&json!("run-finished"))
    );
}

#[test]
fn text_shows_a_failure_and_its_output_indented_under_the_test() {
    let project = cargo_project("run-text", TWO_CRATES, 101);

    let text = stdout_of(&project.pwe(&["test", "run", "--all", "--format", "text"]));

    assert!(
        text.contains("FAILED cargo:crates/beta/src/lib.rs::tests::same_name\n    assertion `left == right` failed: two is two\n"),
        "{text}"
    );
    assert!(text.contains("    expected: 3\n    actual: 2\n"), "{text}");
}
