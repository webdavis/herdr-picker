use std::path::Path;

use pwe_domain::test_engine::{Target, TestEvent, failed_ids};
use pwe_protocol::test_engine::{Hint, TestError, TestErrorCode};

use super::history::{StoredRun, last_finished_run};
use super::run_target::bad_target;

pub(super) struct ResolvedTarget {
    pub(super) recorded: Target,
    pub(super) ran: Target,
    pub(super) runner: Option<String>,
}

pub(super) fn resolved_target(target: Target, runs: &Path) -> Result<ResolvedTarget, TestError> {
    match target {
        Target::Failed => {
            let ids = failed_ids(&last_run(runs)?.events);
            if ids.is_empty() {
                return Err(no_tests("No test failed in the last run."));
            }
            Ok(ResolvedTarget {
                recorded: Target::Failed,
                ran: Target::Ids(ids),
                runner: None,
            })
        }
        Target::Last => {
            let (runner, previous) = previous_target(&last_run(runs)?)?;
            let mut resolved = resolved_target(previous, runs)?;
            resolved.runner = Some(runner);
            Ok(resolved)
        }
        other => Ok(ResolvedTarget {
            recorded: other.clone(),
            ran: other,
            runner: None,
        }),
    }
}

fn last_run(runs: &Path) -> Result<StoredRun, TestError> {
    last_finished_run(runs)
        .ok_or_else(|| no_tests("No finished test run is stored for this folder."))
}

fn previous_target(run: &StoredRun) -> Result<(String, Target), TestError> {
    let started = run.events.iter().find_map(|event| match event {
        TestEvent::RunStarted(started) => Some(started),
        _ => None,
    });
    match started {
        Some(started) if started.target != Target::Last => {
            Ok((started.runner.clone(), started.target.clone()))
        }
        _ => Err(bad_target("The last run's target cannot be repeated.")),
    }
}

fn no_tests(message: &str) -> TestError {
    TestError::new(TestErrorCode::NoTests, message).with_hint(Hint {
        command: Some("pwe test run --all".to_string()),
        ..Hint::default()
    })
}
