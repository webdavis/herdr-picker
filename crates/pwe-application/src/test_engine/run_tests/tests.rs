mod fakes;

use std::cell::Cell;

use pwe_domain::test_engine::{RunStatus, Selection, Target, TestEvent, ToolEnd};

use super::*;
use crate::test_engine::ToolStartError;
use fakes::{FakeClock, FakeTool, Recorded, WordReader, runner};

fn run(
    tool: &mut FakeTool,
    selection: Selection,
) -> (Result<Verdict, RunTestsError>, Vec<TestEvent>) {
    let runner = runner(&[]);
    let request = RunRequest {
        runner: &runner,
        target: Target::All,
        selection,
        cwd: "/project".to_string(),
        repo_root: "/project".to_string(),
        commit: None,
    };
    let mut sink = Recorded::default();
    let verdict = run_tests(
        &request,
        &mut WordReader::default(),
        tool,
        &FakeClock(Cell::new(1_000)),
        &mut sink,
    );
    (verdict, sink.0)
}

fn last_run_finished(events: &[TestEvent]) -> RunFinished {
    match events.last() {
        Some(TestEvent::RunFinished(finished)) => finished.clone(),
        other => panic!("the last event is not run-finished: {other:?}"),
    }
}

#[test]
fn run_finished_is_last_even_when_the_tool_dies_mid_test() {
    let mut tool = FakeTool::printing(
        &["ok passes", "failing dies"],
        ToolEnd::Exited { code: None },
    );

    let (_, events) = run(&mut tool, Selection::All);

    let finished = last_run_finished(&events);
    assert_eq!(
        (
            finished.verdict.status,
            finished.verdict.exit_code,
            finished.tool_exit_code
        ),
        (RunStatus::Failed, 1, None)
    );
    assert_eq!(finished.verdict.counts.failed, 1);
}

#[test]
fn run_started_comes_first_and_names_the_command() {
    let mut tool = FakeTool::printing(&["ok passes"], ToolEnd::Exited { code: Some(0) });

    let (_, events) = run(&mut tool, Selection::All);

    assert!(
        matches!(&events[0], TestEvent::RunStarted(started) if started.argv == ["fake", "test"]),
        "{events:?}"
    );
}

#[test]
fn a_passing_run_finishes_passed_with_exit_0_and_its_duration() {
    let mut tool = FakeTool::printing(&["ok passes"], ToolEnd::Exited { code: Some(0) });

    let (verdict, events) = run(&mut tool, Selection::All);

    let finished = last_run_finished(&events);
    assert_eq!(verdict.map(|verdict| verdict.exit_code), Ok(0));
    assert_eq!(
        (finished.verdict.status, finished.duration_ms),
        (RunStatus::Passed, 750)
    );
}

#[test]
fn a_tool_that_cannot_start_ends_the_run_with_no_event() {
    let mut tool = FakeTool::printing(&[], ToolEnd::Exited { code: Some(0) });
    tool.missing = true;

    let (verdict, events) = run(&mut tool, Selection::All);

    assert_eq!(
        verdict,
        Err(RunTestsError::Start(ToolStartError::Missing(
            "fake".to_string()
        )))
    );
    assert!(events.is_empty(), "{events:?}");
}

#[test]
fn the_tool_runs_the_selected_names_through_the_command_prefix_in_the_folder() {
    let mut tool = FakeTool::printing(&[], ToolEnd::Exited { code: Some(0) });
    let runner = runner(&["mise", "exec", "--"]);
    let request = RunRequest {
        runner: &runner,
        target: Target::All,
        selection: Selection::Names(vec!["a b".to_string(), "c".to_string()]),
        cwd: "/project".to_string(),
        repo_root: "/project".to_string(),
        commit: None,
    };

    let _ = run_tests(
        &request,
        &mut WordReader::default(),
        &mut tool,
        &FakeClock(Cell::new(0)),
        &mut Recorded::default(),
    );

    assert_eq!(
        tool.started,
        vec![(
            ["mise", "exec", "--", "fake", "a b", "c"]
                .map(String::from)
                .to_vec(),
            "/project".to_string()
        )]
    );
}

#[test]
fn a_selection_the_runner_has_no_command_for_is_refused_before_it_starts() {
    let mut tool = FakeTool::printing(&[], ToolEnd::Exited { code: Some(0) });

    let (verdict, events) = run(&mut tool, Selection::File("a.rs".to_string()));

    assert_eq!(
        verdict,
        Err(RunTestsError::Invocation(InvocationError::NoFileCommand))
    );
    assert!(events.is_empty() && tool.started.is_empty());
}
