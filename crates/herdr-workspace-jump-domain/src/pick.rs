mod frame;

pub use frame::{Frame, Row};

use crate::JumpTarget;

/// The keystroke that closes the popup without jumping, beside the escape key.
const CANCEL_CHARACTER: char = 'q';

/// One keystroke the pick popup read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickKey {
    Character(char),
    Escape,
    /// Ctrl-C, which raw mode delivers as a keystroke rather than a signal.
    Interrupt,
    Next,
    Previous,
    Enter,
    Other,
}

/// What the popup does with the keystroke it read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pick<'a> {
    Selected(&'a JumpTarget),
    Cancelled,
    Unbound,
    Changed,
}

#[derive(Debug, Clone)]
pub struct Menu<'a> {
    targets: &'a [JumpTarget],
    cursor: usize,
}

impl<'a> Menu<'a> {
    pub fn new(targets: &'a [JumpTarget]) -> Self {
        Self { targets, cursor: 0 }
    }

    pub fn frame(&self) -> Frame<'a> {
        let rows = self
            .targets
            .iter()
            .enumerate()
            .map(|(row, target)| Row {
                target,
                selected: row == self.cursor,
            })
            .collect();
        Frame { rows }
    }

    pub fn decide(&mut self, key: PickKey) -> Pick<'a> {
        match key {
            PickKey::Escape | PickKey::Interrupt => Pick::Cancelled,
            PickKey::Next => {
                let last = self.targets.len().saturating_sub(1);
                self.cursor = (self.cursor + 1).min(last);
                Pick::Changed
            }
            PickKey::Previous => {
                self.cursor = self.cursor.saturating_sub(1);
                Pick::Changed
            }
            PickKey::Enter => match self.targets.get(self.cursor) {
                Some(target) => Pick::Selected(target),
                None => Pick::Changed,
            },
            PickKey::Character(character) => {
                match self
                    .targets
                    .iter()
                    .find(|target| target.pick_key == character)
                {
                    Some(target) => Pick::Selected(target),
                    None if character == CANCEL_CHARACTER => Pick::Cancelled,
                    None => Pick::Unbound,
                }
            }
            PickKey::Other => Pick::Unbound,
        }
    }
}

#[cfg(test)]
mod tests;
