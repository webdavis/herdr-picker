use pwe_domain::Pane;

use crate::JumpError;

pub trait PanePlaces {
    fn focused_pane(&mut self, rex_session: &str) -> Result<Pane, JumpError>;
}
