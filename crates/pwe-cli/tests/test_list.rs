mod test_project;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use serde_json::{Value, json};
use test_project::{TestProject, exit_code, json_of, lines_of, stdout_of};

const LISTING: &str =
    include_str!("../../pwe-protocol/src/test_engine/libtest/fixtures/cargo-test-list.txt");
const BUILD_FAILED: &str =
    include_str!("../../pwe-protocol/src/test_engine/libtest/fixtures/cargo-test-build-failed.txt");
const IDS: [&str; 5] = [
    "cargo:src/lib.rs::tests::fails",
    "cargo:src/lib.rs::tests::passes",
    "cargo:src/lib.rs::tests::prints",
    "cargo:src/lib.rs::tests::skipped",
    "cargo:tests/it.rs::outside",
];

fn cargo_project(name: &str, output: &str, exit: i32) -> TestProject {
    let project = TestProject::new(name);
    project
        .file("Cargo.toml", "[workspace]\n")
        .fake_tool("cargo", output, exit);
    project
}

fn listing_file(project: &TestProject) -> PathBuf {
    let repo_key = project.project().to_string_lossy().replace('/', "%");
    project
        .state()
        .join("pwe/tests")
        .join(repo_key)
        .join("listings/cargo.jsonl")
}

fn ids_of(tests: &[Value]) -> Vec<&str> {
    tests
        .iter()
        .filter_map(|test| test["id"].as_str())
        .collect()
}

#[test]
fn list_prints_each_test_the_runners_listing_names() {
    let project = cargo_project("list", LISTING, 0);

    let output = project.pwe(&["test", "list", "--format", "json"]);
    let tests = json_of(&stdout_of(&output))["tests"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    assert_eq!(exit_code(&output), Some(0), "{output:?}");
    assert_eq!(ids_of(&tests), IDS);
    assert_eq!(
        tests[0],
        json!({
            "id": IDS[0],
            "runner": "cargo",
            "file": "src/lib.rs",
            "suite": "tests",
            "name": "fails",
            "case": null,
            "line": null,
        })
    );
}

#[test]
fn list_runs_the_runners_listing_command() {
    let project = cargo_project("list-command", LISTING, 0);

    project.pwe(&["test", "list"]);

    assert_eq!(
        project.tool_arguments("cargo"),
        [
            "test",
            "--message-format",
            "json",
            "--",
            "--list",
            "--format",
            "terse"
        ]
    );
}

#[test]
fn list_as_jsonl_prints_one_test_per_line() {
    let project = cargo_project("list-jsonl", LISTING, 0);

    let tests = lines_of(&project.pwe(&["test", "list", "--format", "jsonl"]));

    assert_eq!(ids_of(&tests), IDS);
}

#[test]
fn list_as_text_prints_one_test_id_per_line() {
    let project = cargo_project("list-text", LISTING, 0);

    let text = stdout_of(&project.pwe(&["test", "list", "--format", "text"]));

    assert_eq!(text, format!("{}\n", IDS.join("\n")));
}

#[test]
fn list_saves_the_listing_as_a_private_file_for_its_runner() {
    let project = cargo_project("list-saved", LISTING, 0);

    project.pwe(&["test", "list", "--format", "json"]);
    let saved = fs::read_to_string(listing_file(&project)).unwrap_or_default();
    let mode = fs::metadata(listing_file(&project))
        .map(|metadata| metadata.permissions().mode() & 0o777)
        .unwrap_or_default();

    let tests: Vec<Value> = saved.lines().map(json_of).collect();
    assert_eq!((ids_of(&tests), mode), (IDS.to_vec(), 0o600));
}

#[test]
fn a_failed_build_lists_no_tests_and_exits_3_with_its_diagnostics() {
    let project = cargo_project("list-build-failed", BUILD_FAILED, 101);

    let output = project.pwe(&["test", "list", "--format", "json"]);
    let listing = json_of(&stdout_of(&output));

    assert_eq!(exit_code(&output), Some(3));
    assert_eq!(
        (
            &listing["tests"],
            &listing["diagnostics"][0]["message"],
            &listing["diagnostics"][0]["untrusted"]
        ),
        (&json!([]), &json!("mismatched types"), &json!(true))
    );
    assert!(!listing_file(&project).exists());
}
