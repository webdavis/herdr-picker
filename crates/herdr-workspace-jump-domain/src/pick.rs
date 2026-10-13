mod command;
mod frame;

pub use command::{Command, Mode, Pick};
pub use frame::{Frame, Refusal, Row, SearchLine};

use crate::field::{Edit, TextField};
use crate::fuzzy::{Match, Matcher};
use crate::{JumpTarget, Workspace};

#[derive(Debug, Clone)]
pub struct Menu<'a, M, F> {
    offered: Vec<Offered<'a>>,
    current: Option<&'a JumpTarget>,
    selected_row: usize,
    matcher: M,
    field: F,
    searching: bool,
    refusal: Option<Refusal<'a>>,
}

#[derive(Debug, Clone)]
struct Offered<'a> {
    target: &'a JumpTarget,
    open: bool,
}

impl<'a, M: Matcher, F: TextField> Menu<'a, M, F> {
    pub fn new(
        targets: &'a [JumpTarget],
        live: &[Workspace],
        active_id: Option<&str>,
        matcher: M,
        field: F,
    ) -> Self {
        let active = live
            .iter()
            .find(|workspace| Some(workspace.workspace_id.as_str()) == active_id);
        let current = targets
            .iter()
            .find(|target| active.is_some_and(|workspace| workspace.label == target.label));
        let offered = targets
            .iter()
            .filter(|target| Some(*target) != current)
            .map(|target| Offered {
                target,
                open: live.iter().any(|workspace| workspace.label == target.label),
            })
            .collect();
        Self {
            offered,
            current,
            selected_row: 0,
            matcher,
            field,
            searching: false,
            refusal: None,
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
            .map(|(row, hit)| Row {
                target: hit.item.target,
                selected: row == self.selected_row,
                open: hit.item.open,
                matched: hit.positions,
            })
            .collect();
        Frame {
            search: self.searching.then(|| SearchLine {
                text: self.field.text(),
                cursor: self.field.cursor(),
            }),
            rows,
            total: self.offered.len(),
            refusal: self.refusal,
        }
    }

    pub fn apply(&mut self, command: Command) -> Pick<'a> {
        self.refusal = None;
        match command {
            Command::Cancel => Pick::Cancelled,
            Command::Accept => match self.shown().get(self.selected_row) {
                Some(hit) => Pick::Selected(hit.item.target),
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
                if let Some(current) = self.current.filter(|current| current.pick_key == character)
                {
                    self.refusal = Some(Refusal::AlreadyCurrent(current));
                    return Pick::Pending;
                }
                match self
                    .offered
                    .iter()
                    .find(|offered| offered.target.pick_key == character)
                {
                    Some(offered) => Pick::Selected(offered.target),
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

    fn shown(&self) -> Vec<Match<'_, Offered<'a>>> {
        if self.searching {
            self.matcher
                .rank(&self.field.text(), &self.offered, |offered| {
                    &offered.target.label
                })
        } else {
            self.offered
                .iter()
                .map(|item| Match {
                    item,
                    positions: Vec::new(),
                })
                .collect()
        }
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
