use pwe_domain::test_engine::{
    InvocationError, MatchedRunner, ResultReader, RunFinished, RunStarted, Selection, Target,
    TestEvent, ToolEnd, Verdict, invocation, verdict,
};

use super::{Clock, EventSink, ToolProcess, ToolStartError};

pub struct RunRequest<'a> {
    pub runner: &'a MatchedRunner,
    pub target: Target,
    pub selection: Selection,
    pub cwd: String,
    pub repo_root: String,
    pub commit: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunTestsError {
    Invocation(InvocationError),
    Start(ToolStartError),
}

struct Results<'a> {
    sink: &'a mut dyn EventSink,
    kept_for_verdict: Vec<TestEvent>,
}

impl Results<'_> {
    fn emit(&mut self, events: Vec<TestEvent>) {
        for event in events {
            self.sink.emit(&event);
            if matches!(
                event,
                TestEvent::TestFinished(_) | TestEvent::BuildFailed { .. }
            ) {
                self.kept_for_verdict.push(event);
            }
        }
    }
}

pub fn run_tests(
    request: &RunRequest,
    reader: &mut dyn ResultReader,
    tools: &mut dyn ToolProcess,
    clock: &dyn Clock,
    sink: &mut dyn EventSink,
) -> Result<Verdict, RunTestsError> {
    let argv = command(request)?;
    let started_at = clock.now_ms();
    let mut run = tools
        .start(&argv, &request.cwd)
        .map_err(RunTestsError::Start)?;
    let mut results = Results {
        sink,
        kept_for_verdict: Vec::new(),
    };
    results.emit(vec![run_started(request, argv)]);
    while let Some(line) = run.next_line() {
        results.emit(reader.read_line(&line));
    }
    let end = run.wait();
    results.emit(reader.finish());
    let verdict = verdict(&results.kept_for_verdict, end);
    let duration_ms = clock.now_ms().saturating_sub(started_at);
    results.emit(vec![TestEvent::RunFinished(RunFinished {
        verdict,
        tool_exit_code: match end {
            ToolEnd::Exited { code } => code,
            ToolEnd::Cancelled => None,
        },
        duration_ms,
    })]);
    Ok(verdict)
}

fn command(request: &RunRequest) -> Result<Vec<String>, RunTestsError> {
    let invocation = invocation(&request.runner.spec, &request.selection, None)
        .map_err(RunTestsError::Invocation)?;
    Ok(request
        .runner
        .command_prefix
        .iter()
        .cloned()
        .chain(invocation.argv)
        .collect())
}

fn run_started(request: &RunRequest, argv: Vec<String>) -> TestEvent {
    TestEvent::RunStarted(RunStarted {
        runner: request.runner.spec.name.clone(),
        target: request.target.clone(),
        argv,
        cwd: request.cwd.clone(),
        repo_root: request.repo_root.clone(),
        commit: request.commit.clone(),
    })
}

#[cfg(test)]
mod tests;
