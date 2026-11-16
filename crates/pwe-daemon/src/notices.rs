mod effects;
mod focus;

use std::io::Write;
use std::path::PathBuf;

use pwe_application::{DesktopNotifier, NoticeScreen, PanePlaces, Whereabouts};
use pwe_domain::{Notice, NoticeBoard, NoticeInput, NoticeSettings, RexFocus, Timers};

const NOTICE_CLOCK: &str = "notice-clock";
const ASK_FOR_SESSION: &str = "ask-for-session";

pub struct NoticeParts {
    pub settings: NoticeSettings,
    pub screen: Box<dyn NoticeScreen + Send>,
    pub notifier: Box<dyn DesktopNotifier + Send>,
    pub whereabouts: Box<dyn Whereabouts + Send>,
    pub places: Box<dyn PanePlaces + Send>,
    pub inbox: PathBuf,
    pub log: Box<dyn Write + Send>,
}

pub(crate) struct NoticeDesk {
    board: NoticeBoard,
    parts: NoticeParts,
    timers: Timers,
    focus: RexFocus,
}

impl NoticeDesk {
    pub(crate) fn new(parts: NoticeParts, now: u64) -> Self {
        let mut timers = Timers::default();
        timers.every(NOTICE_CLOCK, 1, now);
        Self {
            board: NoticeBoard::new(parts.settings.clone()),
            parts,
            timers,
            focus: RexFocus::Closed,
        }
    }

    pub(crate) fn apply(&mut self, input: NoticeInput, now: u64) -> Vec<Notice> {
        if let NoticeInput::OwnerMoved(focus) = &input {
            self.focus = focus.clone();
        }
        let effects = self.board.apply(input, now);
        self.run(effects, now)
    }

    pub(crate) fn run_due(&mut self, now: u64) {
        for job in self.timers.due(now) {
            let input = match job.as_str() {
                ASK_FOR_SESSION => NoticeInput::OwnerMoved(self.owner_focus()),
                _ => NoticeInput::TimePassed,
            };
            self.apply(input, now);
        }
    }

    pub(crate) fn next_at(&self) -> Option<u64> {
        self.timers.next_at()
    }

    fn report(&mut self, failure: impl std::fmt::Display) {
        let line = format!("pwe daemon: {failure}\n");
        let _ = self.parts.log.write_all(line.as_bytes());
    }
}

#[cfg(test)]
mod tests;
