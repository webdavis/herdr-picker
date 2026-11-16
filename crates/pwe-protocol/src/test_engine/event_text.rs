use pwe_domain::test_engine::{
    Diagnostic, Failure, RunFinished, RunStatus, TestEvent, TestFinished, TestStatus, UntrustedText,
};

use super::severity_words::severity_name;

const INDENT: &str = "    ";

pub fn event_text(event: &TestEvent) -> String {
    match event {
        TestEvent::RunStarted(run) => format!("Running {} in {}\n", run.argv.join(" "), run.cwd),
        TestEvent::BuildFailed { diagnostics } => {
            let found: String = diagnostics.iter().map(diagnostic_text).collect();
            format!("Build failed:\n{found}")
        }
        TestEvent::TestStarted { .. } => String::new(),
        TestEvent::Output { id, text, .. } => {
            let owner = id
                .as_ref()
                .map_or("the run".to_string(), ToString::to_string);
            format!("Output of {owner}:\n{}", indented(text))
        }
        TestEvent::TestFinished(finished) => test_finished(finished),
        TestEvent::OutputCut { bytes_dropped } => {
            format!("Output cut: {bytes_dropped} more bytes were dropped.\n")
        }
        TestEvent::RunFinished(run) => run_finished(run),
    }
}

fn diagnostic_text(found: &Diagnostic) -> String {
    let message: String = found
        .message
        .text
        .lines()
        .map(|line| format!("{INDENT}{INDENT}{line}\n"))
        .collect();
    format!(
        "{INDENT}{} {}:{}:{}:\n{message}",
        severity_name(found.severity),
        found.file,
        found.line,
        found.column
    )
}

fn test_finished(finished: &TestFinished) -> String {
    let word = match finished.status {
        TestStatus::Passed => "passed",
        TestStatus::Failed => "FAILED",
        TestStatus::Skipped => "skipped",
        TestStatus::Errored => "ERRORED",
    };
    let mut text = format!("{word} {}\n", finished.id);
    if let Some(failure) = &finished.failure {
        text.push_str(&failure_text(failure));
    }
    text
}

fn failure_text(failure: &Failure) -> String {
    let mut text = indented(&failure.message);
    for (label, value) in [("expected", &failure.expected), ("actual", &failure.actual)] {
        if let Some(value) = value {
            text.push_str(&format!("{INDENT}{label}: {}\n", value.text));
        }
    }
    if let (Some(file), Some(line)) = (&failure.file, failure.line) {
        text.push_str(&format!("{INDENT}at {file}:{line}\n"));
    }
    text
}

fn indented(text: &UntrustedText) -> String {
    text.text
        .lines()
        .map(|line| format!("{INDENT}{line}\n"))
        .collect()
}

fn run_finished(run: &RunFinished) -> String {
    let counts = run.verdict.counts;
    let status = match run.verdict.status {
        RunStatus::Passed => "passed",
        RunStatus::Failed => "failed",
        RunStatus::Errored => "errored",
        RunStatus::Cancelled => "cancelled",
    };
    format!(
        "Run {status}: {} passed, {} failed, {} skipped, {} errored in {} ms; exit code {}.\n",
        counts.passed,
        counts.failed,
        counts.skipped,
        counts.errored,
        run.duration_ms,
        run.verdict.exit_code
    )
}
