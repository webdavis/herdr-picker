use pwe_domain::test_engine::{
    Diagnostic, Failure, RunFinished, RunStarted, RunStatus, Stream, Target, TestEvent,
    TestFinished, TestStatus,
};
use serde_json::{Map, Value};

use super::fields::Fields;

pub(super) fn event_fields(event: &TestEvent) -> (&'static str, Map<String, Value>) {
    let (name, fields) = match event {
        TestEvent::RunStarted(run) => ("run-started", run_started(run)),
        TestEvent::BuildFailed { diagnostics } => (
            "build-failed",
            Fields::default().put(
                "diagnostics",
                diagnostics.iter().map(diagnostic).collect::<Vec<_>>(),
            ),
        ),
        TestEvent::TestStarted { id, file, line } => (
            "test-started",
            Fields::default()
                .put("id", id.to_string())
                .put_some("file", file.clone())
                .put_some("line", *line),
        ),
        TestEvent::Output { id, stream, text } => (
            "output",
            Fields::default()
                .put_some("id", id.as_ref().map(ToString::to_string))
                .put("stream", stream_name(*stream))
                .put_untrusted(&[("text", Some(text))]),
        ),
        TestEvent::TestFinished(finished) => ("test-finished", test_finished(finished)),
        TestEvent::OutputCut { bytes_dropped } => (
            "output-cut",
            Fields::default().put("bytes-dropped", *bytes_dropped),
        ),
        TestEvent::RunFinished(run) => ("run-finished", run_finished(run)),
    };
    (name, fields.into_map())
}

fn run_started(run: &RunStarted) -> Fields {
    Fields::default()
        .put("runner", run.runner.clone())
        .put("target", target(&run.target))
        .put("argv", run.argv.clone())
        .put("cwd", run.cwd.clone())
        .put("repo-root", run.repo_root.clone())
        .put_some("commit", run.commit.clone())
}

fn target(target: &Target) -> Fields {
    let (kind, value): (&str, Value) = match target {
        Target::Nearest { file, line } => ("nearest", format!("{file}:{line}").into()),
        Target::File(file) => ("file", file.clone().into()),
        Target::Ids(ids) => (
            "ids",
            ids.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .into(),
        ),
        Target::Failed => ("failed", Value::Null),
        Target::Last => ("last", Value::Null),
        Target::Changed => ("changed", Value::Null),
        Target::All => ("all", Value::Null),
    };
    Fields::default().put("kind", kind).put("value", value)
}

fn diagnostic(diagnostic: &Diagnostic) -> Fields {
    Fields::default()
        .put("file", diagnostic.file.clone())
        .put("line", diagnostic.line)
        .put("column", diagnostic.column)
        .put("severity", diagnostic.severity.clone())
        .put("message", diagnostic.message.clone())
}

fn test_finished(finished: &TestFinished) -> Fields {
    Fields::default()
        .put("id", finished.id.to_string())
        .put("status", test_status_name(finished.status))
        .put_some("duration-ms", finished.duration_ms)
        .put_some("file", finished.file.clone())
        .put_some("line", finished.line)
        .put("attempt", finished.attempt)
        .put_some("failure", finished.failure.as_ref().map(failure))
}

fn failure(failure: &Failure) -> Fields {
    Fields::default()
        .put_some("file", failure.file.clone())
        .put_some("line", failure.line)
        .put_untrusted(&[
            ("message", Some(&failure.message)),
            ("expected", failure.expected.as_ref()),
            ("actual", failure.actual.as_ref()),
        ])
}

fn run_finished(run: &RunFinished) -> Fields {
    let counts = run.verdict.counts;
    Fields::default()
        .put("status", run_status_name(run.verdict.status))
        .put("exit-code", run.verdict.exit_code)
        .put_some("tool-exit-code", run.tool_exit_code)
        .put("duration-ms", run.duration_ms)
        .put(
            "counts",
            Fields::default()
                .put("passed", counts.passed)
                .put("failed", counts.failed)
                .put("skipped", counts.skipped)
                .put("errored", counts.errored),
        )
}

fn stream_name(stream: Stream) -> &'static str {
    match stream {
        Stream::Stdout => "stdout",
        Stream::Stderr => "stderr",
    }
}

fn test_status_name(status: TestStatus) -> &'static str {
    match status {
        TestStatus::Passed => "passed",
        TestStatus::Failed => "failed",
        TestStatus::Skipped => "skipped",
        TestStatus::Errored => "errored",
    }
}

fn run_status_name(status: RunStatus) -> &'static str {
    match status {
        RunStatus::Passed => "passed",
        RunStatus::Failed => "failed",
        RunStatus::Errored => "errored",
        RunStatus::Cancelled => "cancelled",
    }
}
