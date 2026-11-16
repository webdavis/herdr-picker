use std::path::Path;

use pwe_application::JumpError;
use pwe_domain::{HerdrCall, NewPane};
use pwe_protocol::{PaneInfo, Refusal, opened_block};

use super::RexPanes;
use crate::pane_handover::{HandoverError, PaneHost};
use crate::response::jump_error;

impl PaneHost for RexPanes {
    fn pane(&mut self, target: &str) -> Result<PaneInfo, JumpError> {
        RexPanes::pane(self, target)
    }

    fn screen(&mut self, target: &str) -> Result<String, JumpError> {
        RexPanes::screen(self, target)
    }

    fn type_text(&mut self, target: &str, text: &str) -> Result<(), JumpError> {
        self.run(&["send", "-b", target, "--", text]).map(|_| ())
    }

    fn press_enter(&mut self, target: &str) -> Result<(), JumpError> {
        self.run(&["send-key", "-b", target, "enter"]).map(|_| ())
    }

    fn open_pane(
        &mut self,
        beside: &PaneInfo,
        place: NewPane,
        folder: &str,
    ) -> Result<String, JumpError> {
        let side = match place {
            NewPane::Left => "left",
            NewPane::Right => "right",
            NewPane::Above => "above",
            NewPane::Below => "below",
            NewPane::Tab => return self.open_window(folder),
        };
        let beside = beside.pane_id.as_str();
        let answer = self.run(&[
            "split", "-b", beside, "--split", side, "-c", folder, "--json",
        ])?;
        opened_block(&answer).map_err(jump_error)
    }

    fn host_call(&mut self, _call: &HerdrCall, _queue: &Path) -> Result<(), HandoverError> {
        Err(HandoverError::Refused(Refusal::OnlyInHerdr))
    }
}

impl RexPanes {
    fn open_window(&mut self, folder: &str) -> Result<String, JumpError> {
        let mut arguments = vec!["window", "new"];
        if let Some(session) = &self.session {
            arguments.extend(["-s", session.as_str()]);
        }
        arguments.extend(["-c", folder, "--json"]);
        let answer = self.run(&arguments)?;
        opened_block(&answer).map_err(jump_error)
    }
}
