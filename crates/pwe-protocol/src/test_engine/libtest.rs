use pwe_domain::test_engine::{Failure, Stream, TestEvent, TestFinished, TestId, TestStatus};

use super::libtest_failure::read_failure_block;
use super::libtest_name::{doc_test_place, test_id};
use super::libtest_target::{TestTarget, TestTargets};

const RESULT_LINE: &str = "test result:";
const FAILURES_HEADING: &str = "failures:";

enum Section {
    Results,
    Failures,
    Block { name: String, lines: Vec<String> },
    FailureList,
}

pub struct LibtestReader {
    runner: String,
    targets: TestTargets,
    target: TestTarget,
    failed: Vec<String>,
    section: Section,
}

impl LibtestReader {
    pub fn new(runner: &str, repo_root: &str) -> Self {
        LibtestReader {
            runner: runner.to_string(),
            targets: TestTargets::new(repo_root),
            target: TestTarget::Unknown,
            failed: Vec::new(),
            section: Section::Results,
        }
    }

    pub fn read_line(&mut self, line: &str) -> Vec<TestEvent> {
        if let Some(target) = self.targets.read(line) {
            let events = self.close_binary();
            self.target = target;
            return events;
        }
        if line.starts_with(RESULT_LINE) {
            return self.close_binary();
        }
        let in_results = matches!(self.section, Section::Results);
        if let Some(name) = block_name(line).filter(|_| !in_results) {
            let events = self.close_block();
            self.section = Section::Block {
                name: name.to_string(),
                lines: Vec::new(),
            };
            return events;
        }
        if line == FAILURES_HEADING {
            let events = self.close_block();
            self.section = if in_results {
                Section::Failures
            } else {
                Section::FailureList
            };
            return events;
        }
        match &mut self.section {
            Section::Block { lines, .. } => lines.push(line.to_string()),
            Section::Results => return self.result_line(line),
            Section::Failures | Section::FailureList => {}
        }
        Vec::new()
    }

    pub fn finish(&mut self) -> Vec<TestEvent> {
        self.close_binary()
    }

    fn result_line(&mut self, line: &str) -> Vec<TestEvent> {
        let Some((name, outcome)) = line
            .strip_prefix("test ")
            .and_then(|rest| rest.rsplit_once(" ... "))
        else {
            return Vec::new();
        };
        let Some(id) = test_id(&self.runner, &self.target, name) else {
            return Vec::new();
        };
        match outcome {
            "ok" => vec![finished(id, TestStatus::Passed, None)],
            "FAILED" => {
                self.failed.push(name.to_string());
                Vec::new()
            }
            ignored if ignored.starts_with("ignored") => {
                vec![finished(id, TestStatus::Skipped, None)]
            }
            _ => Vec::new(),
        }
    }

    fn close_block(&mut self) -> Vec<TestEvent> {
        let section = std::mem::replace(&mut self.section, Section::Failures);
        let Section::Block { name, lines } = section else {
            self.section = section;
            return Vec::new();
        };
        self.failed.retain(|failed| *failed != name);
        let Some(id) = test_id(&self.runner, &self.target, &name) else {
            return Vec::new();
        };
        let block = read_failure_block(&lines, doc_test_place(&self.target, &name));
        let output = block.output.map(|text| TestEvent::Output {
            id: Some(id.clone()),
            stream: Stream::Stdout,
            text,
        });
        output
            .into_iter()
            .chain([finished(id, TestStatus::Failed, Some(block.failure))])
            .collect()
    }

    fn close_binary(&mut self) -> Vec<TestEvent> {
        let mut events = self.close_block();
        for name in std::mem::take(&mut self.failed) {
            events.extend(
                test_id(&self.runner, &self.target, &name)
                    .map(|id| finished(id, TestStatus::Failed, None)),
            );
        }
        self.section = Section::Results;
        events
    }
}

fn block_name(line: &str) -> Option<&str> {
    line.strip_prefix("---- ")?.strip_suffix(" stdout ----")
}

fn finished(id: TestId, status: TestStatus, failure: Option<Failure>) -> TestEvent {
    TestEvent::TestFinished(TestFinished {
        id,
        status,
        duration_ms: None,
        file: None,
        line: None,
        attempt: 1,
        failure,
    })
}

#[cfg(test)]
mod tests;
