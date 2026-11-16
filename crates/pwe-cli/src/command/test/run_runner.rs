use pwe_application::test_engine::{RunTestsError, ToolStartError};
use pwe_domain::test_engine::{InvocationError, MatchedRunner, Target};
use pwe_protocol::test_engine::{Hint, PARSERS, TestError, TestErrorCode};

use super::run_target::bad_target;

const RUNNERS_COMMAND: &str = "pwe test runners";

pub(super) fn chosen_runner(
    matched: Vec<MatchedRunner>,
    wanted: Option<&str>,
    target: &Target,
) -> Result<MatchedRunner, TestError> {
    let wanted = wanted_name(wanted, target)?;
    let available: Vec<String> = matched
        .iter()
        .map(|runner| runner.spec.name.clone())
        .collect();
    let refusal = |code, message: String| {
        TestError::new(code, message).with_hint(Hint {
            command: Some(RUNNERS_COMMAND.to_string()),
            available: available.clone(),
            ..Hint::default()
        })
    };
    let mut candidates: Vec<MatchedRunner> = matched
        .into_iter()
        .filter(|runner| wanted.as_ref().is_none_or(|name| runner.spec.name == *name))
        .collect();
    match (candidates.len(), wanted) {
        (1, _) => Ok(candidates.remove(0)),
        (0, Some(name)) => Err(refusal(
            TestErrorCode::NoRunner,
            format!("No runner named `{name}` claims this folder."),
        )),
        (0, None) => Err(refusal(
            TestErrorCode::NoRunner,
            "No test runner claims this folder.".to_string(),
        )),
        _ => Err(refusal(
            TestErrorCode::AmbiguousRunner,
            "Several runners claim this folder; name one with --runner.".to_string(),
        )),
    }
}

fn wanted_name(flag: Option<&str>, target: &Target) -> Result<Option<String>, TestError> {
    let mut names: Vec<String> = match target {
        Target::Ids(ids) => ids.iter().map(|id| id.runner.clone()).collect(),
        _ => Vec::new(),
    };
    names.extend(flag.map(str::to_string));
    names.dedup();
    match names.len() {
        0 | 1 => Ok(names.pop()),
        _ => Err(bad_target("One run takes the tests of one runner.")),
    }
}

pub(super) fn unknown_parser(runner: &MatchedRunner) -> TestError {
    TestError::new(
        TestErrorCode::BadRunnerFile,
        format!(
            "The {} runner's output.parser `{}` is not a parser pwe knows.",
            runner.spec.name, runner.spec.output.parser
        ),
    )
    .with_hint(Hint {
        accepted: PARSERS.map(String::from).to_vec(),
        ..Hint::default()
    })
}

pub(super) fn run_refusal(runner: &MatchedRunner, refusal: RunTestsError) -> TestError {
    let name = &runner.spec.name;
    match refusal {
        RunTestsError::Start(start @ ToolStartError::Missing(_)) => TestError::new(
            TestErrorCode::ToolMissing,
            format!("The {name} runner cannot run: {start}."),
        ),
        RunTestsError::Start(start) => {
            TestError::new(TestErrorCode::ToolMissing, start.to_string())
        }
        RunTestsError::Invocation(InvocationError::NoFileCommand) => bad_target(&format!(
            "The {name} runner has no command that runs one file."
        )),
        RunTestsError::Invocation(InvocationError::NoIdsCommand) => bad_target(&format!(
            "The {name} runner has no command that runs chosen tests."
        )),
        RunTestsError::Invocation(InvocationError::Unfilled(value))
        | RunTestsError::Invocation(InvocationError::Unknown(value)) => TestError::new(
            TestErrorCode::BadRunnerFile,
            format!(
                "The {name} runner's [run] command uses `{{{value}}}`, which this target cannot fill."
            ),
        ),
    }
}
