use std::mem;

use super::NoticeBoard;
use super::float::{Float, lowest_free_slot};
use super::input::{NoticeEffect, Raised, RexFocus};
use super::line::notice_line;
use crate::notice::{Level, Notice};

impl NoticeBoard {
    pub(super) fn raise(&mut self, raised: Raised, now: u64) -> Vec<NoticeEffect> {
        if let Some(index) = self.live.iter().position(|live| live.key == raised.key) {
            return self.update(index, raised, now);
        }
        let notice = Notice {
            id: format!("{now}-{}", self.next_number()),
            key: raised.key,
            source: raised.source,
            kind: raised.kind,
            level: raised.level,
            text: raised.text,
            pane: raised.pane,
            tab_label: raised.tab_label,
            raised_at: now,
            updated_at: now,
            resolved_at: None,
            read_at: None,
        };
        self.live.push(notice.clone());
        let mut effects = vec![NoticeEffect::SaveRow(Box::new(notice.clone()))];
        effects.extend(self.present(&notice, now));
        effects
    }

    pub(super) fn present(&mut self, notice: &Notice, now: u64) -> Vec<NoticeEffect> {
        if notice.level == Level::Quiet || self.is_in_view(notice) {
            return Vec::new();
        }
        if let Some(grouped) = self.join_group(notice, now) {
            return vec![grouped];
        }
        self.open_float(
            notice.level,
            &notice.kind,
            vec![notice.id.clone()],
            notice_line(notice),
            now,
        )
    }

    pub(super) fn is_in_view(&self, notice: &Notice) -> bool {
        matches!(&self.focus, RexFocus::Front(pane) if notice.is_about(pane))
    }

    pub(super) fn open_float(
        &mut self,
        level: Level,
        kind: &str,
        members: Vec<String>,
        line: String,
        now: u64,
    ) -> Vec<NoticeEffect> {
        let mut effects = Vec::new();
        if self.floats.len() >= self.settings.most_on_screen {
            let Some(index) = self.oldest_float_no_more_urgent_than(level) else {
                return effects;
            };
            let pushed_off = self.floats.remove(index);
            effects.push(NoticeEffect::Close {
                float: pushed_off.id,
            });
        }
        let seconds = self.settings.seconds_on_screen(level);
        let float = Float {
            id: format!("float-{}", self.next_number()),
            slot: lowest_free_slot(&self.floats),
            level,
            kind: kind.to_string(),
            members,
            shown_at: now,
            until: now + seconds,
        };
        effects.push(NoticeEffect::Show {
            float: float.id.clone(),
            slot: float.slot,
            level,
            line,
            seconds,
        });
        self.floats.push(float);
        effects
    }

    fn oldest_float_no_more_urgent_than(&self, level: Level) -> Option<usize> {
        self.floats
            .iter()
            .enumerate()
            .filter(|(_, float)| float.level.urgency() <= level.urgency())
            .min_by_key(|(_, float)| (float.level.urgency(), float.shown_at))
            .map(|(index, _)| index)
    }

    pub(super) fn expire(&mut self, now: u64) -> Vec<NoticeEffect> {
        let (ended, staying): (Vec<Float>, Vec<Float>) = mem::take(&mut self.floats)
            .into_iter()
            .partition(|float| float.until <= now);
        self.floats = staying;
        ended
            .into_iter()
            .map(|float| NoticeEffect::Close { float: float.id })
            .collect()
    }
}
