use std::fmt;
use std::path::PathBuf;

use pwe_domain::ProgramReport;

pub trait AgentProgram {
    fn sessions(&mut self) -> Result<Vec<ProgramReport>, AgentReadError>;
}

pub trait ProcessEnvironment {
    fn variable(&self, pid: u32, name: &str) -> Option<String>;
}

pub trait OpenFiles {
    fn opened_by(&self, command: &str) -> Vec<OpenFile>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenFile {
    pub pid: u32,
    pub path: PathBuf,
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
