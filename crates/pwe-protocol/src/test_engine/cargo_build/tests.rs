use pwe_domain::test_engine::{Diagnostic, Severity, TestEvent, UntrustedText};

use crate::test_engine::result_reader;

const BUILD_FAILED: &str = include_str!("../libtest/fixtures/cargo-test-build-failed.txt");

fn read_all(output: &str) -> Vec<TestEvent> {
    let mut reader = result_reader("libtest", "cargo", "/work").unwrap();
    let mut events: Vec<TestEvent> = output
        .lines()
        .flat_map(|line| reader.read_line(line))
        .collect();
    events.extend(reader.finish());
    events
}

fn compiler_message(file: &str, level: &str, message: &str) -> String {
    serde_json::json!({
        "reason": "compiler-message",
        "message": {
            "level": level,
            "message": message,
            "spans": [
                {"file_name": "src/other.rs", "line_start": 1, "column_start": 1, "is_primary": false},
                {"file_name": file, "line_start": 7, "column_start": 2, "is_primary": true}
            ]
        }
    })
    .to_string()
}

fn diagnostics_of(lines: &[String]) -> Vec<Diagnostic> {
    match read_all(&lines.join("\n")).as_slice() {
        [TestEvent::BuildFailed { diagnostics }] => diagnostics.clone(),
        other => panic!("not one build-failed event: {other:?}"),
    }
}

const BUILD_FINISHED_FAILED: &str = r#"{"reason":"build-finished","success":false}"#;

#[test]
fn a_failed_build_reports_each_error_once_with_its_place_and_message() {
    assert_eq!(
        read_all(BUILD_FAILED),
        [TestEvent::BuildFailed {
            diagnostics: vec![Diagnostic {
                file: "src/lib.rs".to_string(),
                line: 3,
                column: 5,
                severity: Severity::Error,
                message: UntrustedText::new("mismatched types"),
            }],
        }]
    );
}

#[test]
fn a_compiler_message_is_untrusted_text_with_its_control_characters_stripped() {
    let diagnostics = diagnostics_of(&[
        compiler_message("src/lib.rs", "warning", "unused \u{1b}[31mvariable"),
        BUILD_FINISHED_FAILED.to_string(),
    ]);

    assert_eq!(
        (diagnostics[0].severity, &diagnostics[0].message),
        (
            Severity::Warning,
            &UntrustedText::new("unused \u{1b}[31mvariable")
        )
    );
    assert_eq!(diagnostics[0].message.stripped, 1);
}

#[test]
fn a_diagnostic_in_a_file_under_the_repo_root_is_named_from_the_root() {
    let diagnostics = diagnostics_of(&[
        compiler_message("/work/crates/alpha/src/lib.rs", "error", "oops"),
        compiler_message("/elsewhere/src/lib.rs", "error", "oops"),
        BUILD_FINISHED_FAILED.to_string(),
    ]);

    let files: Vec<&str> = diagnostics
        .iter()
        .map(|found| found.file.as_str())
        .collect();
    assert_eq!(files, ["crates/alpha/src/lib.rs", "/elsewhere/src/lib.rs"]);
}

#[test]
fn a_successful_build_reports_no_diagnostics() {
    let events = read_all(
        &[
            compiler_message("src/lib.rs", "warning", "unused variable"),
            r#"{"reason":"build-finished","success":true}"#.to_string(),
        ]
        .join("\n"),
    );

    assert!(events.is_empty(), "{events:?}");
}

#[test]
fn an_unknown_severity_word_reads_as_other() {
    let diagnostics = diagnostics_of(&[
        compiler_message("src/lib.rs", "failure-note\u{1b}[2J", "oops"),
        BUILD_FINISHED_FAILED.to_string(),
    ]);

    assert_eq!(diagnostics[0].severity, Severity::Other);
}

#[test]
fn a_file_name_with_an_escape_sequence_is_printed_without_it() {
    let diagnostics = diagnostics_of(&[
        compiler_message("src/\u{1b}[2J\nlib.rs", "error", "oops"),
        BUILD_FINISHED_FAILED.to_string(),
    ]);

    assert_eq!(diagnostics[0].file, "src/[2Jlib.rs");
}
