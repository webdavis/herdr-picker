use pwe_domain::Pane;

use crate::JumpError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacedPane {
    pub pane: Pane,
    pub tab_label: String,
}

pub trait PanePlaces {
    fn focused_pane(&mut self, rex_session: &str) -> Result<Pane, JumpError>;
    fn placed_pane(&mut self, block: &str) -> Result<Option<PlacedPane>, JumpError>;
}
