use super::NoticeBoard;
use super::input::{NoticeEffect, Raised};
use crate::notice::Notice;

impl NoticeBoard {
    pub(super) fn update(&mut self, index: usize, raised: Raised, now: u64) -> Vec<NoticeEffect> {
        let notice = &mut self.live[index];
        notice.source = raised.source;
        notice.kind = raised.kind;
        notice.level = raised.level;
        notice.text = raised.text;
        notice.pane = raised.pane;
        notice.tab_label = raised.tab_label;
        notice.updated_at = now;
        let notice = notice.clone();
        let mut effects = vec![NoticeEffect::SaveRow(Box::new(notice.clone()))];
        match self.float_holding(&notice.id) {
            Some(index) if self.floats[index].holds_like(&notice) => {
                effects.push(self.refresh(index, now));
            }
            _ => {
                effects.extend(self.withdraw(&notice.id, now));
                effects.extend(self.present(&notice, now));
            }
        }
        effects
    }

    pub(super) fn resolve(&mut self, key: &str, now: u64) -> Vec<NoticeEffect> {
        let Some(index) = self.live.iter().position(|live| live.key == key) else {
            return Vec::new();
        };
        self.retire(index, now, |notice| notice.resolved_at = Some(now))
    }

    pub(super) fn mark_read(&mut self, id: &str, now: u64) -> Vec<NoticeEffect> {
        let Some(index) = self.live.iter().position(|live| live.id == id) else {
            return Vec::new();
        };
        self.retire(index, now, |notice| notice.read_at = Some(now))
    }

    fn retire(
        &mut self,
        index: usize,
        now: u64,
        mark: impl FnOnce(&mut Notice),
    ) -> Vec<NoticeEffect> {
        let mut notice = self.live.remove(index);
        mark(&mut notice);
        let mut effects = self.withdraw(&notice.id, now);
        effects.insert(0, NoticeEffect::SaveRow(Box::new(notice)));
        effects
    }
}
