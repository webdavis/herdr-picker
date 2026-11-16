use super::*;
use crate::test_engine::{
    Diagnostic, Severity, TestEvent, TestFinished, TestId, TestStatus, UntrustedText,
};

fn finished(name: &str, status: TestStatus) -> TestEvent {
    TestEvent::TestFinished(TestFinished {
        id: format!("cargo:src/lib.rs::tests::{name}")
            .parse::<TestId>()
            .unwrap(),
        status,
        duration_ms: None,
        file: None,
        line: None,
        attempt: 1,
        failure: None,
    })
}

fn exited(code: i32) -> ToolEnd {
    ToolEnd::Exited { code: Some(code) }
}

#[test]
fn a_run_with_one_failed_test_finishes_failed_and_exits_1() {
    let events = [
        finished("passes", TestStatus::Passed),
        finished("fails", TestStatus::Failed),
    ];

    let verdict = verdict(&events, exited(101));

    assert_eq!((verdict.status, verdict.exit_code), (RunStatus::Failed, 1));
}

#[test]
fn a_run_where_every_test_passed_exits_0() {
    let events = [
        finished("passes", TestStatus::Passed),
        finished("skipped", TestStatus::Skipped),
    ];

    let verdict = verdict(&events, exited(0));

    assert_eq!((verdict.status, verdict.exit_code), (RunStatus::Passed, 0));
}

#[test]
fn an_errored_test_fails_the_run() {
    let verdict = verdict(&[finished("crashes", TestStatus::Errored)], exited(101));

    assert_eq!((verdict.status, verdict.exit_code), (RunStatus::Failed, 1));
}

#[test]
fn a_failed_build_errors_the_run_and_exits_3() {
    let events = [TestEvent::BuildFailed {
        diagnostics: vec![Diagnostic {
            file: "src/lib.rs".to_string(),
            line: 3,
            column: 5,
            severity: Severity::Error,
            message: UntrustedText::new("expected `;`"),
        }],
    }];

    let verdict = verdict(&events, exited(101));

    assert_eq!((verdict.status, verdict.exit_code), (RunStatus::Errored, 3));
}

#[test]
fn a_cancelled_run_exits_130_even_after_a_failure() {
    let verdict = verdict(&[finished("fails", TestStatus::Failed)], ToolEnd::Cancelled);

    assert_eq!(
        (verdict.status, verdict.exit_code),
        (RunStatus::Cancelled, 130)
    );
}

#[test]
fn a_tool_that_dies_without_reporting_a_failure_errors_the_run() {
    let events = [finished("passes", TestStatus::Passed)];

    let verdict = verdict(&events, ToolEnd::Exited { code: None });

    assert_eq!((verdict.status, verdict.exit_code), (RunStatus::Errored, 1));
}

#[test]
fn the_counts_hold_one_entry_per_finished_test_by_status() {
    let events = [
        finished("a", TestStatus::Passed),
        finished("b", TestStatus::Passed),
        finished("c", TestStatus::Failed),
        finished("d", TestStatus::Skipped),
        finished("e", TestStatus::Errored),
    ];

    assert_eq!(
        verdict(&events, exited(101)).counts,
        Counts {
            passed: 2,
            failed: 1,
            skipped: 1,
            errored: 1,
        }
    );
}
