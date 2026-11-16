mod test_project;

use serde_json::json;
use test_project::{TestProject, exit_code, lines_of, stdout_of};

const BUILD_FAILED: &str =
    include_str!("../../pwe-protocol/src/test_engine/libtest/fixtures/cargo-test-build-failed.txt");

fn broken_project(name: &str) -> TestProject {
    let project = TestProject::new(name);
    project
        .file("Cargo.toml", "[workspace]\n")
        .fake_tool("cargo", BUILD_FAILED, 101);
    project
}

#[test]
fn a_failed_build_ends_with_build_failed_then_run_finished_errored_and_exits_3() {
    let project = broken_project("build-failed");

    let output = project.pwe(&["test", "run", "--all", "--format", "jsonl"]);
    let events = lines_of(&output);
    let [.., build_failed, last] = events.as_slice() else {
        panic!("too few events: {events:?}");
    };

    assert_eq!(exit_code(&output), Some(3));
    assert_eq!(
        (&build_failed["event"], &build_failed["diagnostics"][0]),
        (
            &json!("build-failed"),
            &json!({
                "file": "src/lib.rs",
                "line": 3,
                "column": 5,
                "severity": "error",
                "message": "mismatched types",
                "untrusted": true,
                "stripped": 0,
            })
        )
    );
    assert_eq!(
        (
            &last["event"],
            &last["status"],
            &last["exit-code"],
            &last["tool-exit-code"]
        ),
        (
            &json!("run-finished"),
            &json!("errored"),
            &json!(3),
            &json!(101)
        )
    );
}

#[test]
fn a_failed_build_as_text_shows_each_compiler_message_indented_under_its_place() {
    let project = broken_project("build-text");

    let text = stdout_of(&project.pwe(&["test", "run", "--all", "--format", "text"]));

    assert!(
        text.contains("Build failed:\n    error src/lib.rs:3:5:\n        mismatched types\n"),
        "{text}"
    );
}
