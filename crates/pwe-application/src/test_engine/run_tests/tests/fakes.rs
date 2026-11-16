use std::cell::Cell;
use std::collections::VecDeque;

use pwe_domain::test_engine::{
    MatchedRunner, OutputSpec, Report, ResultReader, RunCommands, RunnerSpec, RunnerStatus,
    TestEvent, TestFinished, TestStatus, ToolEnd,
};

use crate::test_engine::{Clock, EventSink, ToolProcess, ToolRun, ToolStartError};

pub(super) struct FakeTool {
    lines: Vec<&'static str>,
    end: ToolEnd,
    pub(super) missing: bool,
    pub(super) started: Vec<(Vec<String>, String)>,
}

struct FakeRun {
    lines: VecDeque<&'static str>,
    end: ToolEnd,
}

impl FakeTool {
    pub(super) fn printing(lines: &[&'static str], end: ToolEnd) -> Self {
        FakeTool {
            lines: lines.to_vec(),
            end,
            missing: false,
            started: Vec::new(),
        }
    }
}

impl ToolProcess for FakeTool {
    fn start(&mut self, argv: &[String], cwd: &str) -> Result<Box<dyn ToolRun>, ToolStartError> {
        if self.missing {
            return Err(ToolStartError::Missing(argv[0].clone()));
        }
        self.started.push((argv.to_vec(), cwd.to_string()));
        Ok(Box::new(FakeRun {
            lines: self.lines.iter().copied().collect(),
            end: self.end,
        }))
    }
}

impl ToolRun for FakeRun {
    fn next_line(&mut self) -> Option<String> {
        self.lines.pop_front().map(str::to_string)
    }

    fn wait(&mut self) -> ToolEnd {
        self.end
    }
}

pub(super) struct FakeClock(pub(super) Cell<u64>);

impl Clock for FakeClock {
    fn now_ms(&self) -> u64 {
        let now = self.0.get();
        self.0.set(now + 750);
        now
    }
}

#[derive(Default)]
pub(super) struct Recorded(pub(super) Vec<TestEvent>);

impl EventSink for Recorded {
    fn emit(&mut self, event: &TestEvent) {
        self.0.push(event.clone());
    }
}

#[derive(Default)]
pub(super) struct WordReader {
    pending_failure: Option<String>,
}

impl ResultReader for WordReader {
    fn read_line(&mut self, line: &str) -> Vec<TestEvent> {
        match line.split_once(' ') {
            Some(("ok", name)) => vec![finished(name, TestStatus::Passed)],
            Some(("failing", name)) => {
                self.pending_failure = Some(name.to_string());
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    fn finish(&mut self) -> Vec<TestEvent> {
        self.pending_failure
            .take()
            .map(|name| finished(&name, TestStatus::Failed))
            .into_iter()
            .collect()
    }
}

fn finished(name: &str, status: TestStatus) -> TestEvent {
    TestEvent::TestFinished(TestFinished {
        id: format!("fake:a.rs::{name}").parse().unwrap(),
        status,
        duration_ms: None,
        file: None,
        line: None,
        attempt: 1,
        failure: None,
    })
}

pub(super) fn runner(command_prefix: &[&str]) -> MatchedRunner {
    MatchedRunner {
        spec: RunnerSpec {
            name: "fake".to_string(),
            language: "fake".to_string(),
            status: RunnerStatus::Tested,
            when: "always".to_string(),
            does: "fakes".to_string(),
            detect_files: vec!["fake.toml".to_string()],
            list: None,
            run: RunCommands {
                all: vec!["fake".to_string(), "test".to_string()],
                ids: Some(vec!["fake".to_string(), "{names...}".to_string()]),
                file: None,
            },
            output: OutputSpec {
                parser: "words".to_string(),
                report: Report::Stdout,
            },
        },
        matched_by: "fake.toml".to_string(),
        command_prefix: command_prefix.iter().map(|word| word.to_string()).collect(),
    }
}
