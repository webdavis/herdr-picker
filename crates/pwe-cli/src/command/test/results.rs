use std::env;

use pwe_adapters::test_engine::repo_root;
use pwe_protocol::test_engine::{Hint, TestError, TestErrorCode, event_text};
use serde_json::{Value, json};

use super::flags::read_flags;
use super::history::{StoredRun, last_finished_run, runs_of, stored_run};
use super::output::{Format, chosen_format, encoded, exit_with, print_out, refused};
use crate::command::{CommandError, failed};

const ACCEPTED: [&str; 3] = ["--run", "--last", "--format"];
const UNFINISHED_RUN_EXIT_CODE: i32 = 1;

pub(super) fn results(words: &[&str]) -> Result<(), CommandError> {
    let flags = read_flags(words, &ACCEPTED).map_err(|error| refused(None, &error))?;
    let format = chosen_format(flags.format.as_deref()).map_err(|error| refused(None, &error))?;
    let refuse = |error: TestError| refused(Some(format), &error);
    let runs = runs_of(&repo_root(&env::current_dir().map_err(failed)?));
    let run = match (&flags.run, flags.last) {
        (Some(_), true) => Err(bad_target(
            "pwe test results takes --run or --last, not both.",
        )),
        (Some(run_id), false) => stored_run(&runs, run_id)
            .ok_or_else(|| bad_target(&format!("No stored run has the id `{run_id}`."))),
        (None, _) => last_finished_run(&runs).ok_or_else(no_finished_run),
    }
    .map_err(refuse)?;
    print_out(&replayed(format, &run));
    exit_with(run.finished().map_or(UNFINISHED_RUN_EXIT_CODE, |finished| {
        finished.verdict.exit_code
    }))
}

fn replayed(format: Format, run: &StoredRun) -> String {
    match format {
        Format::Jsonl => run.lines.clone(),
        Format::Text => run.events.iter().map(event_text).collect(),
        Format::Json | Format::Toon => {
            let events: Vec<Value> = run
                .lines
                .lines()
                .filter_map(|line| serde_json::from_str(line).ok())
                .collect();
            encoded(format, &json!({ "events": events }), String::new)
        }
    }
}

fn bad_target(message: &str) -> TestError {
    TestError::new(TestErrorCode::BadTarget, message).with_hint(Hint {
        accepted: ["--run", "--last"].map(String::from).to_vec(),
        ..Hint::default()
    })
}

fn no_finished_run() -> TestError {
    TestError::new(
        TestErrorCode::BadTarget,
        "No finished test run is stored for this folder.",
    )
    .with_hint(Hint {
        command: Some("pwe test run --all".to_string()),
        ..Hint::default()
    })
}
