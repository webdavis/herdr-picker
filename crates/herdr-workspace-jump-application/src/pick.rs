use std::fmt;
use std::io;

use herdr_workspace_jump_domain::{Frame, JumpTarget, Menu, Pick, PickKey};

pub trait KeySource {
    fn next_key(&mut self) -> io::Result<PickKey>;
}

pub trait Screen {
    fn show(&mut self, frame: &Frame<'_>) -> io::Result<()>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickError {
    Show(String),
    ReadKey(String),
}

impl fmt::Display for PickError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Show(detail) => write!(f, "cannot show the menu: {detail}"),
            Self::ReadKey(detail) => write!(f, "cannot read a key: {detail}"),
        }
    }
}

pub fn pick<'a>(
    mut menu: Menu<'a>,
    keys: &mut impl KeySource,
    screen: &mut impl Screen,
) -> Result<Option<&'a JumpTarget>, PickError> {
    loop {
        screen
            .show(&menu.frame())
            .map_err(|failure| PickError::Show(failure.to_string()))?;
        let key = keys
            .next_key()
            .map_err(|failure| PickError::ReadKey(failure.to_string()))?;
        match menu.decide(key) {
            Pick::Selected(target) => return Ok(Some(target)),
            Pick::Cancelled | Pick::Unbound => return Ok(None),
            Pick::Changed => {}
        }
    }
}

#[cfg(test)]
mod tests;
