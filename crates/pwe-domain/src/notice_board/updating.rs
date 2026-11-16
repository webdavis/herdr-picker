use super::NoticeBoard;
use super::input::{NoticeEffect, Raised};
use super::line::notice_line;

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
            Some(index) if self.floats[index].level == notice.level => {
                effects.push(self.refresh(index, now));
            }
            _ => {
                effects.extend(self.withdraw(&notice.id));
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
        let mut effects = self.withdraw(&notice.id);
        effects.insert(0, NoticeEffect::SaveRow(Box::new(notice)));
        effects
    }

    pub(super) fn float_holding(&self, id: &str) -> Option<usize> {
        self.floats
            .iter()
            .position(|float| float.members.iter().any(|member| member == id))
    }

    pub(super) fn withdraw(&mut self, id: &str) -> Vec<NoticeEffect> {
        let Some(index) = self.float_holding(id) else {
            return Vec::new();
        };
        let float = self.floats.remove(index);
        vec![NoticeEffect::Close { float: float.id }]
    }

    fn refresh(&mut self, index: usize, now: u64) -> NoticeEffect {
        let seconds = self.settings.seconds_on_screen(self.floats[index].level);
        self.floats[index].until = now + seconds;
        let float = &self.floats[index];
        NoticeEffect::Relabel {
            float: float.id.clone(),
            line: self.float_line(&float.members),
            seconds,
        }
    }

    fn float_line(&self, members: &[String]) -> String {
        self.live
            .iter()
            .find(|notice| members.contains(&notice.id))
            .map(notice_line)
            .unwrap_or_default()
    }
}
