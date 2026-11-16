use pwe_domain::test_engine::{
    Counts, Diagnostic, Failure, RunFinished, RunStarted, RunStatus, Stream, TestEvent,
    TestFinished, TestStatus, Verdict,
};
use serde_json::Value;

use super::event_fields::{run_status_name, stream_name, test_status_name};
use super::event_values::{Object, read_target};

pub fn read_event_line(line: &str) -> Option<TestEvent> {
    let value: Value = serde_json::from_str(line).ok()?;
    let event = Object(&value);
    Some(match event.text("event")?.as_str() {
        "run-started" => TestEvent::RunStarted(run_started(event)?),
        "build-failed" => TestEvent::BuildFailed {
            diagnostics: event
                .objects("diagnostics")?
                .map(diagnostic)
                .collect::<Option<_>>()?,
        },
        "test-started" => TestEvent::TestStarted {
            id: event.id("id")?,
            file: event.text("file"),
            line: event.number("line"),
        },
        "output" => TestEvent::Output {
            id: event.id("id"),
            stream: [Stream::Stdout, Stream::Stderr]
                .into_iter()
                .find(|stream| Some(stream_name(*stream)) == event.word("stream"))?,
            text: event.untrusted("text")?,
        },
        "test-finished" => TestEvent::TestFinished(test_finished(event)?),
        "output-cut" => TestEvent::OutputCut {
            bytes_dropped: event.count("bytes-dropped")?,
        },
        "run-finished" => TestEvent::RunFinished(run_finished(event)?),
        _ => return None,
    })
}

fn run_started(event: Object) -> Option<RunStarted> {
    Some(RunStarted {
        runner: event.text("runner")?,
        target: read_target(event.object("target")?)?,
        argv: event.words("argv")?,
        cwd: event.text("cwd")?,
        repo_root: event.text("repo-root")?,
        commit: event.text("commit"),
    })
}

fn diagnostic(found: Object) -> Option<Diagnostic> {
    Some(Diagnostic {
        file: found.text("file")?,
        line: found.number("line")?,
        column: found.number("column")?,
        severity: found.text("severity")?,
        message: found.text("message")?,
    })
}

fn test_finished(event: Object) -> Option<TestFinished> {
    Some(TestFinished {
        id: event.id("id")?,
        status: [
            TestStatus::Passed,
            TestStatus::Failed,
            TestStatus::Skipped,
            TestStatus::Errored,
        ]
        .into_iter()
        .find(|status| Some(test_status_name(*status)) == event.word("status"))?,
        duration_ms: event.count("duration-ms"),
        file: event.text("file"),
        line: event.number("line"),
        attempt: event.number("attempt")?,
        failure: match event.object("failure") {
            Some(failure) => Some(read_failure(failure)?),
            None => None,
        },
    })
}

fn read_failure(failure: Object) -> Option<Failure> {
    Some(Failure {
        message: failure.untrusted("message")?,
        file: failure.text("file"),
        line: failure.number("line"),
        expected: failure.untrusted("expected"),
        actual: failure.untrusted("actual"),
    })
}

fn run_finished(event: Object) -> Option<RunFinished> {
    let counts = event.object("counts")?;
    Some(RunFinished {
        verdict: Verdict {
            status: [
                RunStatus::Passed,
                RunStatus::Failed,
                RunStatus::Errored,
                RunStatus::Cancelled,
            ]
            .into_iter()
            .find(|status| Some(run_status_name(*status)) == event.word("status"))?,
            exit_code: i32::try_from(event.0.get("exit-code")?.as_i64()?).ok()?,
            counts: Counts {
                passed: counts.number("passed")?,
                failed: counts.number("failed")?,
                skipped: counts.number("skipped")?,
                errored: counts.number("errored")?,
            },
        },
        tool_exit_code: event
            .0
            .get("tool-exit-code")
            .and_then(Value::as_i64)
            .and_then(|code| i32::try_from(code).ok()),
        duration_ms: event.count("duration-ms")?,
    })
}

#[cfg(test)]
mod tests;
