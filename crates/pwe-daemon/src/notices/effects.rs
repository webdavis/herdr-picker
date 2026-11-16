use pwe_application::NoticeFloat;
use pwe_domain::{Notice, NoticeEffect, RexFocus};

use pwe_adapters::save_inbox_row;

use super::{ASK_FOR_SESSION, NoticeDesk};

impl NoticeDesk {
    pub(super) fn run(&mut self, effects: Vec<NoticeEffect>, now: u64) -> Vec<Notice> {
        let mut waiting = Vec::new();
        for effect in effects {
            match effect {
                NoticeEffect::Waiting(notices) => waiting = notices,
                effect => self.run_one(effect, now),
            }
        }
        waiting
    }

    fn run_one(&mut self, effect: NoticeEffect, now: u64) {
        let outcome = match effect {
            NoticeEffect::Show {
                float,
                slot,
                level,
                line,
                seconds,
            } => self.show(NoticeFloat {
                float,
                rex_session: String::new(),
                slot,
                level,
                line,
                until: now + seconds,
            }),
            NoticeEffect::Relabel { float, line, .. } => self
                .parts
                .screen
                .relabel(&float, &line)
                .map_err(|failure| format!("could not relabel a notice: {failure}")),
            NoticeEffect::Close { float } => self
                .parts
                .screen
                .close(&float)
                .map_err(|failure| format!("could not close a notice: {failure}")),
            NoticeEffect::SaveRow(notice) => save_inbox_row(&self.parts.inbox, &notice)
                .map_err(|failure| format!("could not save an inbox row: {failure}")),
            NoticeEffect::SendToMacos { title, text } => self
                .parts
                .notifier
                .notify(&title, &text)
                .map_err(|failure| format!("could not send a macOS notification: {failure}")),
            NoticeEffect::AskForSession { every_seconds } => {
                self.timers.every(ASK_FOR_SESSION, every_seconds, now);
                Ok(())
            }
            NoticeEffect::StopAskingForSession => {
                self.timers.cancel(ASK_FOR_SESSION);
                Ok(())
            }
            NoticeEffect::JumpTo(_) | NoticeEffect::Waiting(_) => Ok(()),
        };
        if let Err(failure) = outcome {
            self.report(failure);
        }
    }

    fn show(&mut self, mut notice: NoticeFloat) -> Result<(), String> {
        let RexFocus::Front(pane) = &self.focus else {
            return Err("a notice was shown while Rex was not in front".to_string());
        };
        notice.rex_session = pane.rex_session.clone();
        self.parts
            .screen
            .show(&notice)
            .map_err(|failure| format!("could not show a notice: {failure}"))
    }
}
