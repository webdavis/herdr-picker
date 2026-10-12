mod frame;

pub use frame::{Frame, Row, SearchLine};

use crate::JumpTarget;
use crate::fuzzy::Matcher;

/// The keystroke that closes the popup without jumping, beside the escape key.
const CANCEL_CHARACTER: char = 'q';

pub(crate) const SEARCH_CHARACTER: char = '/';

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
    Backspace,
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
pub struct Menu<'a, M> {
    targets: &'a [JumpTarget],
    current: Option<&'a JumpTarget>,
    selected_row: usize,
    matcher: M,
    query: String,
    searching: bool,
}

impl<'a, M: Matcher> Menu<'a, M> {
    pub fn new(targets: &'a [JumpTarget], current_label: Option<&str>, matcher: M) -> Self {
        Self {
            targets,
            current: targets
                .iter()
                .find(|target| Some(target.label.as_str()) == current_label),
            selected_row: 0,
            matcher,
            query: String::new(),
            searching: false,
        }
    }

    pub fn frame(&self) -> Frame<'a> {
        let rows = self
            .shown()
            .into_iter()
            .enumerate()
            .map(|(row, target)| Row {
                target,
                selected: row == self.selected_row,
                current: self.current == Some(target),
            })
            .collect();
        Frame {
            search: self.searching.then(|| SearchLine {
                text: self.query.clone(),
            }),
            rows,
            total: self.targets.len(),
        }
    }

    pub fn decide(&mut self, key: PickKey) -> Pick<'a> {
        match key {
            PickKey::Escape | PickKey::Interrupt => Pick::Cancelled,
            PickKey::Enter => match self.shown().get(self.selected_row) {
                Some(&target) => Pick::Selected(target),
                None => Pick::Pending,
            },
            PickKey::Next => {
                let last = self.shown().len().saturating_sub(1);
                self.selected_row = (self.selected_row + 1).min(last);
                Pick::Pending
            }
            PickKey::Previous => {
                self.selected_row = self.selected_row.saturating_sub(1);
                Pick::Pending
            }
            PickKey::Character(SEARCH_CHARACTER) if !self.searching => {
                self.searching = true;
                self.selected_row = 0;
                Pick::Pending
            }
            PickKey::Character(character) if !self.searching => {
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
            PickKey::Backspace | PickKey::Other if !self.searching => Pick::Unbound,
            PickKey::Character(character) => {
                self.query.push(character);
                self.selected_row = 0;
                Pick::Pending
            }
            PickKey::Backspace if self.query.is_empty() => {
                self.searching = false;
                self.selected_row = 0;
                Pick::Pending
            }
            PickKey::Backspace => {
                self.query.pop();
                self.selected_row = 0;
                Pick::Pending
            }
            PickKey::Other => Pick::Pending,
        }
    }

    fn shown(&self) -> Vec<&'a JumpTarget> {
        if self.searching {
            self.matcher
                .rank(&self.query, self.targets, |target| &target.label)
        } else {
            self.targets.iter().collect()
        }
    }
}

#[cfg(test)]
mod tests;
