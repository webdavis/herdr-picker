use super::join::joined;
use super::{AgentSession, AgentState, PaneStatus, ProgramReport};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentChange {
    Started(AgentSession),
    Changed(AgentSession),
    Ended(AgentSession),
}

#[derive(Debug, Clone, Default)]
pub struct AgentBook {
    sessions: Vec<AgentSession>,
}

impl AgentBook {
    pub fn sessions(&self) -> &[AgentSession] {
        &self.sessions
    }

    pub fn newest_state_first(&self) -> Vec<AgentSession> {
        let mut sessions = self.sessions.clone();
        sessions.sort_by_key(|session| std::cmp::Reverse(session.state_since));
        sessions
    }

    pub fn update(
        &mut self,
        reports: &[ProgramReport],
        panes: &[PaneStatus],
        now: u64,
    ) -> Vec<AgentChange> {
        let current: Vec<AgentSession> = reports
            .iter()
            .map(|report| joined(report, panes, self.earlier(report), now))
            .collect();
        let mut changes: Vec<AgentChange> = current
            .iter()
            .filter_map(|session| self.change_to(session))
            .collect();
        changes.extend(
            self.sessions
                .iter()
                .filter(|earlier| {
                    !reports
                        .iter()
                        .any(|report| earlier.is_the_same_session_as(report))
                })
                .map(|earlier| AgentChange::Ended(ended(earlier, now))),
        );
        self.sessions = current;
        changes
    }

    fn earlier(&self, report: &ProgramReport) -> Option<&AgentSession> {
        self.sessions
            .iter()
            .find(|session| session.is_the_same_session_as(report))
    }

    fn change_to(&self, session: &AgentSession) -> Option<AgentChange> {
        let earlier = self.sessions.iter().find(|earlier| {
            earlier.program == session.program && earlier.session_id == session.session_id
        });
        match earlier {
            None => Some(AgentChange::Started(session.clone())),
            Some(earlier) if earlier != session => Some(AgentChange::Changed(session.clone())),
            Some(_) => None,
        }
    }
}

fn ended(session: &AgentSession, now: u64) -> AgentSession {
    AgentSession {
        state: AgentState::Ended,
        waiting_for: None,
        state_since: now,
        ..session.clone()
    }
}
