use pwe_application::JumpError;
use pwe_protocol::{PaneFront, rex_pane_front};

use super::RexPanes;
use crate::response::jump_error;

impl RexPanes {
    pub fn front(&mut self, target: &str) -> Result<PaneFront, JumpError> {
        let process = self.run(&["block", "call", "-b", target, "process"])?;
        rex_pane_front(&process).map_err(jump_error)
    }
}
