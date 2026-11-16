mod float;
mod grouping;
mod input;
mod line;
mod settings;
mod showing;
mod updating;

pub use input::{NoticeEffect, NoticeInput, Raised, RexFocus};
pub use settings::NoticeSettings;

use crate::notice::Notice;
use float::Float;

#[derive(Debug, Clone)]
pub struct NoticeBoard {
    settings: NoticeSettings,
    live: Vec<Notice>,
    floats: Vec<Float>,
    focus: RexFocus,
    numbers: u64,
}

impl NoticeBoard {
    pub fn new(settings: NoticeSettings) -> Self {
        Self {
            settings,
            live: Vec::new(),
            floats: Vec::new(),
            focus: RexFocus::Closed,
            numbers: 0,
        }
    }

    pub fn apply(&mut self, input: NoticeInput, now: u64) -> Vec<NoticeEffect> {
        match input {
            NoticeInput::Raise(raised) => self.raise(raised, now),
            NoticeInput::OwnerMoved(focus) => {
                self.focus = focus;
                Vec::new()
            }
            NoticeInput::Resolve { key } => self.resolve(&key, now),
            NoticeInput::TimePassed => self.expire(now),
        }
    }

    fn next_number(&mut self) -> u64 {
        self.numbers += 1;
        self.numbers
    }
}

#[cfg(test)]
mod tests;
