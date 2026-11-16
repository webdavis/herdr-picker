use super::NoticeBoard;
use super::input::NoticeEffect;
use super::line::{group_line, notice_line};
use crate::notice::Notice;

impl NoticeBoard {
    pub(super) fn join_group(&mut self, notice: &Notice, now: u64) -> Option<NoticeEffect> {
        let index = self
            .floats
            .iter()
            .position(|float| float.holds_like(notice))?;
        self.floats[index].members.push(notice.id.clone());
        Some(self.refresh(index, now))
    }

    pub(super) fn float_holding(&self, id: &str) -> Option<usize> {
        self.floats
            .iter()
            .position(|float| float.members.iter().any(|member| member == id))
    }

    pub(super) fn withdraw(&mut self, id: &str, now: u64) -> Vec<NoticeEffect> {
        let Some(index) = self.float_holding(id) else {
            return Vec::new();
        };
        self.floats[index].members.retain(|member| member != id);
        let float = &self.floats[index];
        if float.members.is_empty() {
            return vec![NoticeEffect::Close {
                float: self.floats.remove(index).id,
            }];
        }
        vec![NoticeEffect::Relabel {
            float: float.id.clone(),
            line: self.float_line(&float.members, now),
            seconds: float.until.saturating_sub(now),
        }]
    }

    pub(super) fn refresh(&mut self, index: usize, now: u64) -> NoticeEffect {
        let seconds = self.settings.seconds_on_screen(self.floats[index].level);
        self.floats[index].until = now + seconds;
        let float = &self.floats[index];
        NoticeEffect::Relabel {
            float: float.id.clone(),
            line: self.float_line(&float.members, now),
            seconds,
        }
    }

    fn float_line(&self, members: &[String], now: u64) -> String {
        let notices: Vec<&Notice> = self
            .live
            .iter()
            .filter(|notice| members.contains(&notice.id))
            .collect();
        match notices.as_slice() {
            [] => String::new(),
            [only] => notice_line(only, now),
            [first, ..] => group_line(notices.len(), &first.kind, first.level),
        }
    }
}
