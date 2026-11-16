use std::path::Path;

use pwe_application::JumpError;
use pwe_domain::{HerdrCall, NewPane};
use pwe_protocol::PaneInfo;

use super::HandoverError;

pub trait PaneHost {
    fn pane(&mut self, target: &str) -> Result<PaneInfo, JumpError>;
    fn screen(&mut self, target: &str) -> Result<String, JumpError>;
    fn type_text(&mut self, target: &str, text: &str) -> Result<(), JumpError>;
    fn press_enter(&mut self, target: &str) -> Result<(), JumpError>;
    fn open_pane(
        &mut self,
        beside: &PaneInfo,
        place: NewPane,
        folder: &str,
    ) -> Result<String, JumpError>;
    fn host_call(&mut self, call: &HerdrCall, queue: &Path) -> Result<(), HandoverError>;
}
