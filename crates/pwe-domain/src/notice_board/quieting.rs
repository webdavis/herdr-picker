use std::mem;

use super::NoticeBoard;
use super::input::NoticeEffect;

const EVERY_SOURCE: &str = "*";

#[derive(Debug, Clone)]
pub(super) struct Mute {
    source: String,
    until: Option<u64>,
}

impl Mute {
    fn is_active(&self, now: u64) -> bool {
        self.until.is_none_or(|until| now < until)
    }

    fn silences(&self, source: &str, now: u64) -> bool {
        (self.source == source || self.source == EVERY_SOURCE) && self.is_active(now)
    }
}

impl NoticeBoard {
    pub(super) fn hide(&mut self) -> Vec<NoticeEffect> {
        mem::take(&mut self.floats)
            .into_iter()
            .map(|float| NoticeEffect::Close { float: float.id })
            .collect()
    }

    pub(super) fn mute(
        &mut self,
        source: String,
        until: Option<u64>,
        now: u64,
    ) -> Vec<NoticeEffect> {
        self.mutes
            .retain(|mute| mute.source != source && mute.is_active(now));
        self.mutes.push(Mute { source, until });
        let silenced: Vec<String> = self
            .live
            .iter()
            .filter(|notice| self.is_muted(&notice.source, now))
            .map(|notice| notice.id.clone())
            .collect();
        silenced
            .iter()
            .flat_map(|id| self.withdraw(id, now))
            .collect()
    }

    pub(super) fn unmute(&mut self, source: &str) -> Vec<NoticeEffect> {
        self.mutes.retain(|mute| mute.source != source);
        Vec::new()
    }

    pub(super) fn is_muted(&self, source: &str, now: u64) -> bool {
        self.mutes.iter().any(|mute| mute.silences(source, now))
    }
}
