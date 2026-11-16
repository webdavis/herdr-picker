mod float;
mod grouping;
mod input;
mod line;
mod moving;
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
    focused_since: u64,
    asking_for_session: bool,
    numbers: u64,
}

impl NoticeBoard {
    pub fn new(settings: NoticeSettings) -> Self {
        Self {
            settings,
            live: Vec::new(),
            floats: Vec::new(),
            focus: RexFocus::Closed,
            focused_since: 0,
            asking_for_session: false,
            numbers: 0,
        }
    }

    pub fn apply(&mut self, input: NoticeInput, now: u64) -> Vec<NoticeEffect> {
        let mut effects = match input {
            NoticeInput::Raise(raised) => self.raise(raised, now),
            NoticeInput::OwnerMoved(focus) => self.owner_moved(focus, now),
            NoticeInput::Resolve { key } => self.resolve(&key, now),
            NoticeInput::TimePassed => [self.expire(now), self.mark_seen(now)].concat(),
            NoticeInput::ListWaiting => vec![NoticeEffect::Waiting(self.waiting())],
        };
        effects.extend(self.follow_waiting());
        effects
    }

    fn next_number(&mut self) -> u64 {
        self.numbers += 1;
        self.numbers
    }
}

#[cfg(test)]
mod tests;
