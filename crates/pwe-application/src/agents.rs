use std::fmt;
use std::path::PathBuf;

use pwe_domain::{AgentBook, AgentSession, PaneStatus, ProgramReport};

pub trait AgentProgram {
    fn sessions(&mut self) -> Result<Vec<ProgramReport>, AgentReadError>;
}

pub trait PaneStatuses {
    fn pane_statuses(&mut self) -> Result<Vec<PaneStatus>, AgentReadError>;
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AgentSources {
    pub reports: Vec<ProgramReport>,
    pub panes: Vec<PaneStatus>,
    pub refusals: Vec<AgentReadError>,
}

pub fn read_agent_sources(
    programs: &mut [&mut dyn AgentProgram],
    rex_panes: &mut [&mut dyn PaneStatuses],
) -> AgentSources {
    let mut sources = AgentSources::default();
    for program in programs.iter_mut() {
        match program.sessions() {
            Ok(sessions) => sources.reports.extend(sessions),
            Err(refusal) => sources.refusals.push(refusal),
        }
    }
    for source in rex_panes.iter_mut() {
        match source.pane_statuses() {
            Ok(statuses) => sources.panes.extend(statuses),
            Err(refusal) => sources.refusals.push(refusal),
        }
    }
    sources
}

pub fn agent_sessions(
    programs: &mut [&mut dyn AgentProgram],
    rex_panes: &mut [&mut dyn PaneStatuses],
    now: u64,
) -> (Vec<AgentSession>, Vec<AgentReadError>) {
    let sources = read_agent_sources(programs, rex_panes);
    (joined_sessions(&sources, now), sources.refusals)
}

pub fn joined_sessions(sources: &AgentSources, now: u64) -> Vec<AgentSession> {
    let mut book = AgentBook::default();
    book.update(&sources.reports, &sources.panes, now);
    book.newest_state_first()
}

#[cfg(test)]
mod tests;
