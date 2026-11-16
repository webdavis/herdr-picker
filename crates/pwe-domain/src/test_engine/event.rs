use super::{TestId, UntrustedText, Verdict};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Errored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub severity: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    pub message: UntrustedText,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub expected: Option<UntrustedText>,
    pub actual: Option<UntrustedText>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestFinished {
    pub id: TestId,
    pub status: TestStatus,
    pub duration_ms: Option<u64>,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub attempt: u32,
    pub failure: Option<Failure>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunFinished {
    pub verdict: Verdict,
    pub tool_exit_code: Option<i32>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestEvent {
    BuildFailed {
        diagnostics: Vec<Diagnostic>,
    },
    TestStarted {
        id: TestId,
        file: Option<String>,
        line: Option<u32>,
    },
    Output {
        id: Option<TestId>,
        stream: Stream,
        text: UntrustedText,
    },
    TestFinished(TestFinished),
    OutputCut {
        bytes_dropped: u64,
    },
    RunFinished(RunFinished),
}
