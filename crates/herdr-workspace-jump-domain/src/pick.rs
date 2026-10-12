mod command;
mod frame;

pub use command::{Command, Mode, Pick};
pub use frame::{Frame, Row, SearchLine};

use crate::JumpTarget;
use crate::field::Edit;
use crate::fuzzy::Matcher;

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

    pub fn mode(&self) -> Mode {
        if self.searching {
            Mode::Searching
        } else {
            Mode::Browsing
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

    pub fn apply(&mut self, command: Command) -> Pick<'a> {
        match command {
            Command::Cancel => Pick::Cancelled,
            Command::Accept => match self.shown().get(self.selected_row) {
                Some(&target) => Pick::Selected(target),
                None => Pick::Pending,
            },
            Command::Next => {
                let last = self.shown().len().saturating_sub(1);
                self.selected_row = (self.selected_row + 1).min(last);
                Pick::Pending
            }
            Command::Previous => {
                self.selected_row = self.selected_row.saturating_sub(1);
                Pick::Pending
            }
            Command::Key(character) => {
                match self
                    .targets
                    .iter()
                    .find(|target| target.pick_key == character)
                {
                    Some(target) => Pick::Selected(target),
                    None => Pick::Unbound,
                }
            }
            Command::Search => {
                self.searching = true;
                self.selected_row = 0;
                Pick::Pending
            }
            Command::Unbound | Command::Erase | Command::Edit(_) if !self.searching => {
                Pick::Unbound
            }
            Command::Unbound => Pick::Pending,
            Command::Erase if self.query.is_empty() => {
                self.searching = false;
                self.selected_row = 0;
                Pick::Pending
            }
            Command::Erase => {
                self.query.pop();
                self.selected_row = 0;
                Pick::Pending
            }
            Command::Edit(edit) => self.edit(edit),
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

    fn edit(&mut self, edit: Edit) -> Pick<'a> {
        let Edit::Insert(character) = edit;
        self.query.push(character);
        self.selected_row = 0;
        Pick::Pending
    }
}

#[cfg(test)]
mod tests;
