mod book;
mod join;
mod words;

pub use book::{AgentBook, AgentChange};
pub use words::{AgentState, ProgramName, StateFrom, WaitingKind};

use crate::notice::Pane;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaitingFor {
    pub kind: WaitingKind,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramReport {
    pub program: ProgramName,
    pub session_id: String,
    pub name: Option<String>,
    pub folder: Option<String>,
    pub account: Option<String>,
    pub pid: Option<u32>,
    pub state: AgentState,
    pub waiting_for: Option<WaitingFor>,
    pub state_since: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneStatus {
    pub pane: Pane,
    pub program: Option<ProgramName>,
    pub pid: Option<u32>,
    pub folder: Option<String>,
    pub state: AgentState,
    pub waiting_for: Option<WaitingFor>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSession {
    pub program: ProgramName,
    pub session_id: String,
    pub name: Option<String>,
    pub folder: Option<String>,
    pub account: Option<String>,
    pub pid: Option<u32>,
    pub pane: Option<Pane>,
    pub state: AgentState,
    pub waiting_for: Option<WaitingFor>,
    pub state_since: u64,
    pub state_from: StateFrom,
}

impl AgentSession {
    fn is_the_same_session_as(&self, report: &ProgramReport) -> bool {
        self.program == report.program && self.session_id == report.session_id
    }
}

#[cfg(test)]
mod tests;
