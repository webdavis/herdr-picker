use pwe_domain::test_engine::{
    ListCommand, OutputSpec, Report, RunCommands, RunnerSpec, RunnerStatus,
};

use super::*;

const FILE: &str = "runners/cargo.toml";

const CARGO: &str = r#"
name = "cargo"
language = "rust"
status = "tested"
when = "The folder has a Cargo.toml."
does = "Runs cargo test across the workspace."

[detect]
files = ["Cargo.toml"]

[list]
argv = ["cargo", "test", "--workspace", "--", "--list", "--format", "terse"]
parser = "libtest-list"

[run]
all = ["cargo", "test", "--workspace", "--no-fail-fast"]
ids = ["cargo", "test", "--workspace", "--no-fail-fast", "--", "--exact", "{names...}"]

[output]
parser = "libtest"
report = "stdout"
"#;

fn words(text: &[&str]) -> Vec<String> {
    text.iter().map(|word| word.to_string()).collect()
}

fn without(line: &str) -> String {
    CARGO
        .lines()
        .filter(|candidate| !candidate.starts_with(line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn replaced(line: &str, with: &str) -> String {
    CARGO.replace(line, with)
}

#[test]
fn a_runner_file_without_a_name_is_refused_naming_its_file_and_field() {
    assert_eq!(
        read_runner_file(FILE, &without("name =")),
        Err(RunnerFileError::Missing {
            file: FILE.to_string(),
            field: "name",
        })
    );
}

#[test]
fn a_complete_runner_file_becomes_its_runner() {
    assert_eq!(
        read_runner_file(FILE, CARGO),
        Ok(RunnerSpec {
            name: "cargo".to_string(),
            language: "rust".to_string(),
            status: RunnerStatus::Tested,
            when: "The folder has a Cargo.toml.".to_string(),
            does: "Runs cargo test across the workspace.".to_string(),
            detect_files: words(&["Cargo.toml"]),
            list: Some(ListCommand {
                argv: words(&[
                    "cargo",
                    "test",
                    "--workspace",
                    "--",
                    "--list",
                    "--format",
                    "terse"
                ]),
                parser: "libtest-list".to_string(),
            }),
            run: RunCommands {
                all: words(&["cargo", "test", "--workspace", "--no-fail-fast"]),
                ids: Some(words(&[
                    "cargo",
                    "test",
                    "--workspace",
                    "--no-fail-fast",
                    "--",
                    "--exact",
                    "{names...}",
                ])),
                file: None,
            },
            output: OutputSpec {
                parser: "libtest".to_string(),
                report: Report::Stdout,
            },
        })
    );
}

#[test]
fn a_missing_field_inside_a_table_is_named_with_its_table() {
    assert_eq!(
        read_runner_file(FILE, &without("all =")),
        Err(RunnerFileError::Missing {
            file: FILE.to_string(),
            field: "run.all",
        })
    );
}

#[test]
fn an_empty_command_is_refused() {
    assert_eq!(
        read_runner_file(
            FILE,
            &replaced(
                r#"all = ["cargo", "test", "--workspace", "--no-fail-fast"]"#,
                "all = []"
            )
        ),
        Err(RunnerFileError::Empty {
            file: FILE.to_string(),
            field: "run.all",
        })
    );
}

#[test]
fn a_status_outside_tested_and_untested_is_refused_with_the_accepted_values() {
    assert_eq!(
        read_runner_file(
            FILE,
            &replaced(r#"status = "tested""#, r#"status = "maybe""#)
        ),
        Err(RunnerFileError::NotOneOf {
            file: FILE.to_string(),
            field: "status",
            value: "maybe".to_string(),
            accepted: &["tested", "untested"],
        })
    );
}

#[test]
fn a_report_outside_stdout_and_file_is_refused() {
    let refused = read_runner_file(
        FILE,
        &replaced(r#"report = "stdout""#, r#"report = "pipe""#),
    );

    assert!(matches!(
        refused,
        Err(RunnerFileError::NotOneOf {
            field: "output.report",
            ..
        })
    ));
}

#[test]
fn a_name_that_could_break_a_test_id_is_refused() {
    assert_eq!(
        read_runner_file(
            FILE,
            &replaced(r#"name = "cargo""#, r#"name = "cargo:unit""#)
        ),
        Err(RunnerFileError::NotKebabCase {
            file: FILE.to_string(),
            field: "name",
            value: "cargo:unit".to_string(),
        })
    );
}

#[test]
fn a_misspelled_field_is_refused_naming_it() {
    let refused = read_runner_file(FILE, &replaced("[detect]", "[detect]\nfiels = []"));

    assert!(matches!(
        &refused,
        Err(RunnerFileError::Unreadable { file, reason }) if file == FILE && reason.contains("fiels")
    ));
}

#[test]
fn every_error_message_names_the_file_and_the_field() {
    let message = read_runner_file(FILE, &without("does ="))
        .unwrap_err()
        .to_string();

    assert!(
        message.contains(FILE) && message.contains("does"),
        "{message}"
    );
}
