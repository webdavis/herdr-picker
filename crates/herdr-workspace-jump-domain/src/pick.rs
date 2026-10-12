use std::fmt::Write;

use crate::JumpTarget;

/// The keystroke that closes the popup without jumping, beside the escape key.
const CANCEL_CHARACTER: char = 'q';

const BOLD_BLUE: &str = "\x1b[1;34m";
const FAINT: &str = "\x1b[2m";
const GREY_BAR: &str = "\x1b[48;5;236m";
const RESET: &str = "\x1b[0m";
const LEGEND: &str = "ctrl+n/p move  enter jump  esc close";

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

    pub fn render(&self, width: usize) -> String {
        let mut menu = String::new();
        for (row, target) in self.targets.iter().enumerate() {
            let key = format!("{:>3}", target.pick_key);
            let label = format!("  {}", target.label);
            let _ = if row == self.cursor {
                let bar = pad(&label, width.saturating_sub(4));
                writeln!(
                    menu,
                    "{GREY_BAR} {BOLD_BLUE}{key}{RESET}{GREY_BAR}{bar}{RESET}"
                )
            } else {
                writeln!(menu, " {BOLD_BLUE}{key}{RESET}{label}")
            };
        }
        let _ = write!(menu, "\n{FAINT} {LEGEND}{RESET}");
        menu
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

fn pad(text: &str, width: usize) -> String {
    format!("{text:width$}")
}

#[cfg(test)]
mod tests;
