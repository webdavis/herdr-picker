use pwe_application::{JumpError, PanePlaces, PlacedPane};
use pwe_domain::Pane;
use pwe_protocol::{focused_pane_in_view, placed_pane_in_view, rex_session_ids, session_request};

use super::RexPanes;
use crate::response::jump_error;

impl RexPanes {
    fn view(&self, rex_session: &str) -> Result<String, JumpError> {
        self.run(&["api", "call", "session.view", &session_request(rex_session)])
    }
}

impl PanePlaces for RexPanes {
    fn focused_pane(&mut self, rex_session: &str) -> Result<Pane, JumpError> {
        focused_pane_in_view(rex_session, &self.view(rex_session)?).map_err(jump_error)
    }

    fn placed_pane(&mut self, block: &str) -> Result<Option<PlacedPane>, JumpError> {
        let sessions = rex_session_ids(&self.run(&["ls", "--json"])?).map_err(jump_error)?;
        for rex_session in sessions {
            let view = self.view(&rex_session)?;
            if let Some((pane, tab_label)) =
                placed_pane_in_view(&rex_session, &view, block).map_err(jump_error)?
            {
                return Ok(Some(PlacedPane { pane, tab_label }));
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests;
