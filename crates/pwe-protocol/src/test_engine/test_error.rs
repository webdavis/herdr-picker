use serde_json::{Map, Value, json};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestErrorCode {
    NoRunner,
    AmbiguousRunner,
    NoTests,
    BadTarget,
    BadFlag,
    BadRunnerFile,
    ToolMissing,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Hint {
    pub command: Option<String>,
    pub accepted: Vec<String>,
    pub did_you_mean: Option<String>,
    pub available: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestError {
    pub code: TestErrorCode,
    pub message: String,
    pub hint: Box<Hint>,
}

impl TestError {
    pub fn new(code: TestErrorCode, message: impl Into<String>) -> Self {
        TestError {
            code,
            message: message.into(),
            hint: Box::default(),
        }
    }

    pub fn with_hint(mut self, hint: Hint) -> Self {
        self.hint = Box::new(hint);
        self
    }

    pub fn json(&self) -> Value {
        let mut hint = Map::new();
        if let Some(command) = &self.hint.command {
            hint.insert("command".to_string(), command.clone().into());
        }
        if !self.hint.accepted.is_empty() {
            hint.insert("accepted".to_string(), self.hint.accepted.clone().into());
        }
        if let Some(guess) = &self.hint.did_you_mean {
            hint.insert("did-you-mean".to_string(), guess.clone().into());
        }
        if !self.hint.available.is_empty() {
            hint.insert("available".to_string(), self.hint.available.clone().into());
        }
        json!({ "error": { "code": code_name(self.code), "message": self.message, "hint": hint } })
    }

    pub fn text(&self) -> String {
        match &self.hint.command {
            Some(command) => format!("{}\n  Try: {command}\n", self.message),
            None => format!("{}\n", self.message),
        }
    }
}

fn code_name(code: TestErrorCode) -> &'static str {
    match code {
        TestErrorCode::NoRunner => "NO_RUNNER",
        TestErrorCode::AmbiguousRunner => "AMBIGUOUS_RUNNER",
        TestErrorCode::NoTests => "NO_TESTS",
        TestErrorCode::BadTarget => "BAD_TARGET",
        TestErrorCode::BadFlag => "BAD_FLAG",
        TestErrorCode::BadRunnerFile => "BAD_RUNNER_FILE",
        TestErrorCode::ToolMissing => "TOOL_MISSING",
    }
}
