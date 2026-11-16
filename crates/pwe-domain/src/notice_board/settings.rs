use crate::notice::Level;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoticeSettings {
    pub finished_seconds: u64,
    pub needs_you_seconds: u64,
    pub most_on_screen: usize,
}

impl Default for NoticeSettings {
    fn default() -> Self {
        Self {
            finished_seconds: 6,
            needs_you_seconds: 10,
            most_on_screen: 3,
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
