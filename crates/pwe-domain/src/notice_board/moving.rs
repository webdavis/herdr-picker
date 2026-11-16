use super::NoticeBoard;
use super::input::{NoticeEffect, RexFocus};
use super::line::reminder_line;
use crate::notice::{Level, Notice};

impl NoticeBoard {
    pub(super) fn owner_moved(&mut self, focus: RexFocus, now: u64) -> Vec<NoticeEffect> {
        let switched = switched_session_or_tab(&self.focus, &focus);
        if self.focus != focus {
            self.focused_since = now;
        }
        self.focus = focus;
        if switched {
            self.mention_what_waits(now)
        } else {
            Vec::new()
        }
    }

    pub(super) fn waiting(&self) -> Vec<Notice> {
        let mut waiting: Vec<Notice> = self
            .live
            .iter()
            .rev()
            .filter(|notice| notice.level == Level::NeedsYou)
            .cloned()
            .collect();
        waiting.sort_by_key(|notice| std::cmp::Reverse(notice.updated_at));
        waiting
    }

    pub(super) fn mark_seen(&mut self, now: u64) -> Vec<NoticeEffect> {
        let RexFocus::Front(pane) = &self.focus else {
            return Vec::new();
        };
        let seen: Vec<String> = self
            .live
            .iter()
            .filter(|notice| notice.level == Level::Finished && notice.is_about(pane))
            .filter(|notice| {
                now >= self.focused_since.max(notice.updated_at) + self.settings.seen_after_seconds
            })
            .map(|notice| notice.id.clone())
            .collect();
        seen.iter().flat_map(|id| self.mark_read(id, now)).collect()
    }

    pub(super) fn follow_waiting(&mut self) -> Option<NoticeEffect> {
        let waits = self
            .live
            .iter()
            .any(|notice| notice.level == Level::NeedsYou);
        if waits == self.asking_for_session {
            return None;
        }
        self.asking_for_session = waits;
        Some(if waits {
            NoticeEffect::AskForSession {
                every_seconds: self.settings.ask_session_every_seconds,
            }
        } else {
            NoticeEffect::StopAskingForSession
        })
    }

    fn mention_what_waits(&mut self, now: u64) -> Vec<NoticeEffect> {
        let waiting = self.waiting();
        let elsewhere: Vec<&Notice> = waiting
            .iter()
            .filter(|notice| !self.is_in_view(notice) && !self.is_muted(&notice.source, now))
            .collect();
        let Some(line) = reminder_line(&elsewhere) else {
            return Vec::new();
        };
        let Some(index) = self
            .floats
            .iter()
            .position(|float| float.members.is_empty())
        else {
            return self.open_float(Level::NeedsYou, "", Vec::new(), line, now);
        };
        let seconds = self.settings.seconds_on_screen(Level::NeedsYou);
        self.floats[index].until = now + seconds;
        vec![NoticeEffect::Relabel {
            float: self.floats[index].id.clone(),
            line,
            seconds,
        }]
    }
}

fn switched_session_or_tab(before: &RexFocus, after: &RexFocus) -> bool {
    matches!(
        (before, after),
        (RexFocus::Front(before), RexFocus::Front(after))
            if (&before.rex_session, &before.tab) != (&after.rex_session, &after.tab)
    )
}
