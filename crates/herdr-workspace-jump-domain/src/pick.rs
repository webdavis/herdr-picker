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
    Pending,
}

#[derive(Debug, Clone)]
pub struct Menu<'a> {
    targets: &'a [JumpTarget],
    current: Option<&'a JumpTarget>,
    selected_row: usize,
}

impl<'a> Menu<'a> {
    pub fn new(targets: &'a [JumpTarget], current_label: Option<&str>) -> Self {
        Self {
            targets,
            current: targets
                .iter()
                .find(|target| Some(target.label.as_str()) == current_label),
            selected_row: 0,
        }
    }

    pub fn frame(&self) -> Frame<'a> {
        let rows = self
            .targets
            .iter()
            .enumerate()
            .map(|(row, target)| Row {
                target,
                selected: row == self.selected_row,
                current: self.current == Some(target),
            })
            .collect();
        Frame { rows }
    }

    pub fn decide(&mut self, key: PickKey) -> Pick<'a> {
        match key {
            PickKey::Escape | PickKey::Interrupt => Pick::Cancelled,
            PickKey::Next => {
                let last = self.targets.len().saturating_sub(1);
                self.selected_row = (self.selected_row + 1).min(last);
                Pick::Pending
            }
            PickKey::Previous => {
                self.selected_row = self.selected_row.saturating_sub(1);
                Pick::Pending
            }
            PickKey::Enter => match self.targets.get(self.selected_row) {
                Some(target) => Pick::Selected(target),
                None => Pick::Pending,
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
