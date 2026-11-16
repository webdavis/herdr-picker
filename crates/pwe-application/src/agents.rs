use std::fmt;
use std::path::PathBuf;

use pwe_domain::{AgentBook, AgentSession, ProgramReport};

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

pub fn agent_sessions(
    programs: &mut [&mut dyn AgentProgram],
    now: u64,
) -> (Vec<AgentSession>, Vec<AgentReadError>) {
    let mut reports = Vec::new();
    let mut refusals = Vec::new();
    for program in programs.iter_mut() {
        match program.sessions() {
            Ok(sessions) => reports.extend(sessions),
            Err(refusal) => refusals.push(refusal),
        }
    }
    let mut book = AgentBook::default();
    book.update(&reports, &[], now);
    (book.newest_state_first(), refusals)
}

#[cfg(test)]
mod tests;
