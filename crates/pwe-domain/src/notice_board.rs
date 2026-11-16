mod float;
mod grouping;
mod input;
mod jumping;
mod line;
mod moving;
mod quieting;
mod settings;
mod showing;
mod updating;

pub use input::{NoticeEffect, NoticeInput, Raised, RexFocus};
pub use settings::NoticeSettings;

use crate::notice::Notice;
use float::Float;
use quieting::Mute;

#[derive(Debug, Clone)]
pub struct NoticeBoard {
    settings: NoticeSettings,
    live: Vec<Notice>,
    floats: Vec<Float>,
    focus: RexFocus,
    focused_since: u64,
    mutes: Vec<Mute>,
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
            mutes: Vec::new(),
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
            NoticeInput::Hide => self.hide(),
            NoticeInput::Mute { source, until } => self.mute(source, until, now),
            NoticeInput::Unmute { source } => self.unmute(&source),
            NoticeInput::Dismiss { id } | NoticeInput::Read { id } => self.mark_read(&id, now),
            NoticeInput::Jump { id } => self.jump(id.as_deref(), now),
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
