use pwe_domain::test_engine::{
    Counts, Failure, RunFinished, RunStarted, RunStatus, Stream, Target, TestEvent, TestFinished,
    TestId, TestStatus, UntrustedText, Verdict,
};
use serde_json::{Value, json};

use super::*;

const TIME: &str = "2026-11-16T08:15:00Z";

fn id() -> TestId {
    "cargo:src/lib.rs::tests::fails".parse().unwrap()
}

fn started() -> TestEvent {
    TestEvent::RunStarted(RunStarted {
        runner: "cargo".to_string(),
        target: Target::Ids(vec![id()]),
        argv: vec!["cargo".to_string(), "test".to_string()],
        cwd: "/work/probe".to_string(),
        repo_root: "/work/probe".to_string(),
        commit: Some("b3e4f19".to_string()),
    })
}

fn written(lines: &mut EventLines, event: &TestEvent) -> Value {
    let line = lines.line(TIME, event);
    assert!(!line.contains('\n'), "one event per line: {line}");
    serde_json::from_str(&line).unwrap()
}

#[test]
fn the_first_event_carries_the_format_version() {
    let mut lines = EventLines::new("20261116T081500Z-3f9a");

    assert_eq!(
        written(&mut lines, &started()),
        json!({
            "event": "run-started",
            "v": 1,
            "run-id": "20261116T081500Z-3f9a",
            "sequence": 1,
            "time": TIME,
            "runner": "cargo",
            "target": {"kind": "ids", "value": ["cargo:src/lib.rs::tests::fails"]},
            "argv": ["cargo", "test"],
            "cwd": "/work/probe",
            "repo-root": "/work/probe",
            "commit": "b3e4f19",
        })
    );
}

#[test]
fn later_events_count_up_without_the_version() {
    let mut lines = EventLines::new("run");
    written(&mut lines, &started());

    let second = written(&mut lines, &TestEvent::OutputCut { bytes_dropped: 9 });

    assert_eq!(
        (
            second.get("v"),
            &second["sequence"],
            &second["bytes-dropped"]
        ),
        (None, &json!(2), &json!(9))
    );
}

#[test]
fn a_failed_test_carries_its_failure_marked_untrusted() {
    let mut lines = EventLines::new("run");
    let event = TestEvent::TestFinished(TestFinished {
        id: id(),
        status: TestStatus::Failed,
        duration_ms: None,
        file: Some("src/lib.rs".to_string()),
        line: Some(5),
        attempt: 1,
        failure: Some(Failure {
            message: UntrustedText::new("two is two\u{1b}"),
            file: Some("src/lib.rs".to_string()),
            line: Some(5),
            expected: Some(UntrustedText::new("3")),
            actual: Some(UntrustedText::new("2")),
        }),
    });

    let value = written(&mut lines, &event);

    assert_eq!(
        (&value["event"], &value["status"], &value["failure"]),
        (
            &json!("test-finished"),
            &json!("failed"),
            &json!({
                "message": "two is two",
                "file": "src/lib.rs",
                "line": 5,
                "expected": "3",
                "actual": "2",
                "untrusted": true,
                "stripped": 1,
            })
        )
    );
}

#[test]
fn fields_the_tool_did_not_report_are_left_out() {
    let mut lines = EventLines::new("run");
    let event = TestEvent::TestStarted {
        id: id(),
        file: None,
        line: None,
    };

    let value = written(&mut lines, &event);

    assert_eq!(
        (value.get("file"), value.get("line"), &value["id"]),
        (None, None, &json!("cargo:src/lib.rs::tests::fails"))
    );
}

#[test]
fn output_is_marked_untrusted_with_its_stripped_count() {
    let mut lines = EventLines::new("run");
    let event = TestEvent::Output {
        id: Some(id()),
        stream: Stream::Stdout,
        text: UntrustedText::new("hello\u{7}\n"),
    };

    let value = written(&mut lines, &event);

    assert_eq!(
        (
            &value["stream"],
            &value["text"],
            &value["untrusted"],
            &value["stripped"]
        ),
        (&json!("stdout"), &json!("hello\n"), &json!(true), &json!(1))
    );
}

#[test]
fn the_run_finished_event_carries_the_verdict_and_counts() {
    let mut lines = EventLines::new("run");
    let event = TestEvent::RunFinished(RunFinished {
        verdict: Verdict {
            status: RunStatus::Failed,
            exit_code: 1,
            counts: Counts {
                passed: 2,
                failed: 1,
                skipped: 1,
                errored: 0,
            },
        },
        tool_exit_code: Some(101),
        duration_ms: 1200,
    });

    let value = written(&mut lines, &event);

    assert_eq!(
        (
            &value["status"],
            &value["exit-code"],
            &value["tool-exit-code"],
            &value["duration-ms"],
            &value["counts"]
        ),
        (
            &json!("failed"),
            &json!(1),
            &json!(101),
            &json!(1200),
            &json!({"passed": 2, "failed": 1, "skipped": 1, "errored": 0})
        )
    );
}

#[test]
fn a_nearest_target_names_its_file_and_line() {
    let mut lines = EventLines::new("run");
    let TestEvent::RunStarted(mut run) = started() else {
        unreachable!()
    };
    run.target = Target::Nearest {
        file: "src/lib.rs".to_string(),
        line: 5,
    };

    let value = written(&mut lines, &TestEvent::RunStarted(run));

    assert_eq!(
        value["target"],
        json!({"kind": "nearest", "value": "src/lib.rs:5"})
    );
}
