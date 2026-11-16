use std::fmt;

use pwe_domain::test_engine::{TestEvent, ToolEnd};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolStartError {
    Missing(String),
    Failed { program: String, reason: String },
}

impl fmt::Display for ToolStartError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing(program) => write!(formatter, "{program} was not found on PATH"),
            Self::Failed { program, reason } => {
                write!(formatter, "{program} could not start: {reason}")
            }
        }
    }
}

pub trait ToolProcess {
    fn start(&mut self, argv: &[String], cwd: &str) -> Result<Box<dyn ToolRun>, ToolStartError>;
}

pub trait ToolRun {
    fn next_line(&mut self) -> Option<String>;
    fn wait(&mut self) -> ToolEnd;
}

pub trait Clock {
    fn now_ms(&self) -> u64;
}

pub trait EventSink {
    fn emit(&mut self, event: &TestEvent);
}
