use pwe_application::{AgentReadError, JumpError, PaneStatuses};
use pwe_domain::PaneStatus;
use pwe_protocol::{
    ProtocolError, rex_pane_status, rex_session_ids, rex_status_record, rex_terminal_panes,
};

use super::{REX_PANES_SOURCE, RexPanes};
use crate::response::jump_error;

impl PaneStatuses for RexPanes {
    fn pane_statuses(&mut self) -> Result<Vec<PaneStatus>, AgentReadError> {
        let mut statuses = Vec::new();
        for session in rex_session_ids(&self.ask(&["ls", "--json"])?).map_err(unreadable)? {
            let blocks = self.ask(&["block", "ls", "-s", &session, "--json"])?;
            for pane in rex_terminal_panes(&session, &blocks).map_err(unreadable)? {
                let status = self.ask(&["block", "call", "-b", &pane.block, "program_status"])?;
                let Some(record) = rex_status_record(&status).map_err(unreadable)? else {
                    continue;
                };
                let process = self.ask(&["block", "call", "-b", &pane.block, "process"])?;
                statuses.push(rex_pane_status(pane, &record, &process).map_err(unreadable)?);
            }
        }
        Ok(statuses)
    }
}

impl RexPanes {
    fn ask(&self, arguments: &[&str]) -> Result<String, AgentReadError> {
        self.run(arguments).map_err(rex_refused)
    }
}

fn rex_refused(failure: JumpError) -> AgentReadError {
    AgentReadError {
        program: REX_PANES_SOURCE.to_string(),
        detail: failure.to_string(),
    }
}

fn unreadable(failure: ProtocolError) -> AgentReadError {
    rex_refused(jump_error(failure))
}

#[cfg(test)]
mod tests;
