use pwe_domain::{Level, Pane};

use crate::JumpError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoticeFloat {
    pub float: String,
    pub rex_session: String,
    pub slot: usize,
    pub level: Level,
    pub line: String,
    pub until: u64,
}

pub trait NoticeScreen {
    fn show(&mut self, notice: &NoticeFloat) -> Result<(), JumpError>;
    fn relabel(&mut self, float: &str, line: &str) -> Result<(), JumpError>;
    fn close(&mut self, float: &str) -> Result<(), JumpError>;
    fn jump(&mut self, pane: &Pane) -> Result<(), JumpError>;
}
