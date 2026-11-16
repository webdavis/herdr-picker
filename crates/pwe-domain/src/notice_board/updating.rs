use super::NoticeBoard;
use super::input::{NoticeEffect, Raised};

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
        let mut notice = self.live.remove(index);
        notice.resolved_at = Some(now);
        let mut effects = self.withdraw(&notice.id, now);
        effects.insert(0, NoticeEffect::SaveRow(Box::new(notice)));
        effects
    }
}
