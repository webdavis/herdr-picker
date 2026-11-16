use std::fmt;

use pwe_domain::ProgramReport;

pub trait AgentProgram {
    fn sessions(&mut self) -> Result<Vec<ProgramReport>, AgentReadError>;
}

pub trait ProcessEnvironment {
    fn variable(&self, pid: u32, name: &str) -> Option<String>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentReadError {
    pub program: String,
    pub detail: String,
}

impl fmt::Display for AgentReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "could not read {} sessions: {}",
            self.program, self.detail
        )
    }
}
