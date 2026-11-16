use crate::notice::Level;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoticeSettings {
    pub finished_seconds: u64,
    pub needs_you_seconds: u64,
    pub most_on_screen: usize,
    pub seen_after_seconds: u64,
    pub ask_session_every_seconds: u64,
}

impl Default for NoticeSettings {
    fn default() -> Self {
        Self {
            finished_seconds: 6,
            needs_you_seconds: 10,
            most_on_screen: 3,
            seen_after_seconds: 1,
            ask_session_every_seconds: 2,
        }
    }
}

impl NoticeSettings {
    pub(crate) fn seconds_on_screen(&self, level: Level) -> u64 {
        match level {
            Level::NeedsYou => self.needs_you_seconds,
            Level::Finished | Level::Quiet => self.finished_seconds,
        }
    }
}
