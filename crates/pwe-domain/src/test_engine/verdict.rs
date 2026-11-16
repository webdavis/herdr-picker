use super::{TestEvent, TestStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunStatus {
    Passed,
    Failed,
    Errored,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolEnd {
    Exited { code: Option<i32> },
    Cancelled,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counts {
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub errored: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Verdict {
    pub status: RunStatus,
    pub exit_code: i32,
    pub counts: Counts,
}

const PASSED_EXIT_CODE: i32 = 0;
const TESTS_FAILED_EXIT_CODE: i32 = 1;
const BUILD_FAILED_EXIT_CODE: i32 = 3;
const CANCELLED_EXIT_CODE: i32 = 130;

pub fn verdict(events: &[TestEvent], end: ToolEnd) -> Verdict {
    let counts = count(events);
    let build_failed = events
        .iter()
        .any(|event| matches!(event, TestEvent::BuildFailed { .. }));
    let (status, exit_code) = match end {
        ToolEnd::Cancelled => (RunStatus::Cancelled, CANCELLED_EXIT_CODE),
        _ if build_failed => (RunStatus::Errored, BUILD_FAILED_EXIT_CODE),
        _ if counts.failed + counts.errored > 0 => (RunStatus::Failed, TESTS_FAILED_EXIT_CODE),
        ToolEnd::Exited { code: Some(0) } => (RunStatus::Passed, PASSED_EXIT_CODE),
        ToolEnd::Exited { .. } => (RunStatus::Errored, TESTS_FAILED_EXIT_CODE),
    };
    Verdict {
        status,
        exit_code,
        counts,
    }
}

fn count(events: &[TestEvent]) -> Counts {
    events.iter().fold(Counts::default(), |mut counts, event| {
        if let TestEvent::TestFinished(finished) = event {
            match finished.status {
                TestStatus::Passed => counts.passed += 1,
                TestStatus::Failed => counts.failed += 1,
                TestStatus::Skipped => counts.skipped += 1,
                TestStatus::Errored => counts.errored += 1,
            }
        }
        counts
    })
}

#[cfg(test)]
mod tests;
