use pwe_domain::test_engine::{MatchedRunner, RunnerStatus};
use serde_json::{Value, json};

use super::RunnerFileError;

pub fn runner_listing_json(runners: &[MatchedRunner], broken: &[RunnerFileError]) -> Value {
    let mut listing = json!({ "runners": runners.iter().map(runner_json).collect::<Vec<_>>() });
    if !broken.is_empty() {
        listing["broken-runner-files"] = broken.iter().map(broken_json).collect();
    }
    listing
}

pub fn runner_listing_text(runners: &[MatchedRunner], broken: &[RunnerFileError]) -> String {
    let mut text = String::new();
    if runners.is_empty() {
        text.push_str("No test runner claims this folder.\n");
    }
    for runner in runners {
        text.push_str(&format!(
            "{} ({}, {}): matched by {}\n",
            runner.spec.name,
            runner.spec.language,
            status_name(runner.spec.status),
            runner.matched_by
        ));
        if !runner.command_prefix.is_empty() {
            text.push_str(&format!(
                "  runs through {}\n",
                runner.command_prefix.join(" ")
            ));
        }
    }
    for error in broken {
        text.push_str(&format!("Broken runner file: {error}\n"));
    }
    text
}

fn runner_json(runner: &MatchedRunner) -> Value {
    json!({
        "name": runner.spec.name,
        "language": runner.spec.language,
        "status": status_name(runner.spec.status),
        "matched-by": runner.matched_by,
        "command-prefix": runner.command_prefix,
    })
}

fn broken_json(error: &RunnerFileError) -> Value {
    json!({
        "file": error.file(),
        "field": error.field(),
        "message": error.to_string(),
    })
}

fn status_name(status: RunnerStatus) -> &'static str {
    match status {
        RunnerStatus::Tested => "tested",
        RunnerStatus::Untested => "untested",
    }
}
