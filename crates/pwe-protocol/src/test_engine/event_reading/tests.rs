use pwe_domain::test_engine::{
    Counts, Diagnostic, Failure, RunFinished, RunStarted, RunStatus, Severity, Stream, Target,
    TestEvent, TestFinished, TestId, TestStatus, UntrustedText, Verdict,
};

use super::*;
use crate::test_engine::EventLines;

fn id() -> TestId {
    "cargo:src/lib.rs::tests::fails[case one]".parse().unwrap()
}

fn read_back(event: TestEvent) -> (TestEvent, Option<TestEvent>) {
    let line = EventLines::new("20261116T081500Z-3f9a").line("2026-11-16T08:15:00Z", &event);
    (event, read_event_line(&line))
}

fn assert_reads_back(event: TestEvent) {
    let (written, read) = read_back(event);
    assert_eq!(read, Some(written));
}

fn started(target: Target) -> TestEvent {
    TestEvent::RunStarted(RunStarted {
        runner: "cargo".to_string(),
        target,
        argv: vec!["cargo".to_string(), "test".to_string()],
        cwd: "/work/probe".to_string(),
        repo_root: "/work/probe".to_string(),
        commit: Some("b3e4f19".to_string()),
    })
}

#[test]
fn a_stored_run_started_reads_back_with_each_kind_of_target() {
    for target in [
        Target::All,
        Target::Failed,
        Target::Last,
        Target::Changed,
        Target::File("src/lib.rs".to_string()),
        Target::Ids(vec![id()]),
        Target::Nearest {
            file: "src/a:b.rs".to_string(),
            line: 7,
        },
    ] {
        assert_reads_back(started(target));
    }
}

#[test]
fn a_stored_failed_test_reads_back_with_its_failure() {
    assert_reads_back(TestEvent::TestFinished(TestFinished {
        id: id(),
        status: TestStatus::Failed,
        duration_ms: Some(12),
        file: Some("src/lib.rs".to_string()),
        line: Some(5),
        attempt: 2,
        failure: Some(Failure {
            message: UntrustedText::new("two is two"),
            file: Some("src/lib.rs".to_string()),
            line: Some(5),
            expected: Some(UntrustedText::new("3")),
            actual: None,
        }),
    }));
}

#[test]
fn every_other_stored_event_reads_back_as_it_was_written() {
    for event in [
        TestEvent::BuildFailed {
            diagnostics: vec![Diagnostic {
                file: "src/lib.rs".to_string(),
                line: 3,
                column: 9,
                severity: Severity::Error,
                message: UntrustedText::new("mismatched types"),
            }],
        },
        TestEvent::TestStarted {
            id: id(),
            file: None,
            line: Some(4),
        },
        TestEvent::Output {
            id: None,
            stream: Stream::Stderr,
            text: UntrustedText::new("hello\n"),
        },
        TestEvent::OutputCut { bytes_dropped: 9 },
        TestEvent::RunFinished(RunFinished {
            verdict: Verdict {
                status: RunStatus::Cancelled,
                exit_code: 130,
                counts: Counts {
                    passed: 2,
                    failed: 1,
                    skipped: 1,
                    errored: 3,
                },
            },
            tool_exit_code: None,
            duration_ms: 1200,
        }),
    ] {
        assert_reads_back(event);
    }
}

#[test]
fn a_line_that_is_not_a_known_event_reads_as_nothing() {
    for line in [
        "",
        "not json",
        r#"{"event":"future-event"}"#,
        r#"{"event":"output-cut"}"#,
    ] {
        assert_eq!(read_event_line(line), None, "{line}");
    }
}
