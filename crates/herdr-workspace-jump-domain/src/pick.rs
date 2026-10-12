mod command;
mod frame;

pub use command::{Command, Mode, Pick};
pub use frame::{Frame, Row, SearchLine};

use crate::JumpTarget;
use crate::field::{Edit, TextField};
use crate::fuzzy::Matcher;

#[derive(Debug, Clone)]
pub struct Menu<'a, M, F> {
    targets: &'a [JumpTarget],
    current: Option<&'a JumpTarget>,
    selected_row: usize,
    matcher: M,
    field: F,
    searching: bool,
}

impl<'a, M: Matcher, F: TextField> Menu<'a, M, F> {
    pub fn new(
        targets: &'a [JumpTarget],
        current_label: Option<&str>,
        matcher: M,
        field: F,
    ) -> Self {
        Self {
            targets,
            current: targets
                .iter()
                .find(|target| Some(target.label.as_str()) == current_label),
            selected_row: 0,
            matcher,
            field,
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
            })
            .collect();
        Frame {
            search: self.searching.then(|| SearchLine {
                text: self.field.text(),
                cursor: self.field.cursor(),
            }),
            rows,
            total: self.offered().count(),
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
            Command::Erase if self.field.is_empty() => {
                self.searching = false;
                self.selected_row = 0;
                Pick::Pending
            }
            Command::Erase => self.edit(Edit::DeleteBackward),
            Command::Edit(edit) => self.edit(edit),
        }
    }

    fn shown(&self) -> Vec<&'a JumpTarget> {
        if self.searching {
            self.matcher
                .rank(&self.field.text(), self.targets, |target| &target.label)
                .into_iter()
                .filter(|target| Some(*target) != self.current)
                .collect()
        } else {
            self.offered().collect()
        }
    }

    fn offered(&self) -> impl Iterator<Item = &'a JumpTarget> {
        self.targets
            .iter()
            .filter(move |target| Some(*target) != self.current)
    }

    fn edit(&mut self, edit: Edit) -> Pick<'a> {
        let before = self.field.text();
        self.field.apply(edit);
        if self.field.text() != before {
            self.selected_row = 0;
        }
        Pick::Pending
    }
}

#[cfg(test)]
mod tests;
