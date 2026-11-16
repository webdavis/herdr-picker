use std::path::PathBuf;
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use pwe_adapters::RexPanes;
use pwe_domain::RecordedPane;

use super::pass::reopen_recorded;

const ROUND_PAUSE: Duration = Duration::from_millis(500);
const ROUNDS_BEFORE_GIVING_UP: usize = 60;

pub(crate) struct ReopenRunner {
    rex_program: PathBuf,
    running: Option<(Sender<()>, JoinHandle<()>)>,
}

impl ReopenRunner {
    pub(crate) fn new(rex_program: PathBuf) -> Self {
        Self {
            rex_program,
            running: None,
        }
    }

    pub(crate) fn reopen(&mut self, records: Vec<RecordedPane>) {
        self.finish();
        let (stop_asked, stop_heard) = mpsc::channel::<()>();
        let mut panes = RexPanes::at(self.rex_program.clone(), None);
        let pass = thread::spawn(move || {
            let wait_a_moment =
                || stop_heard.recv_timeout(ROUND_PAUSE) == Err(RecvTimeoutError::Timeout);
            reopen_recorded(records, &mut panes, wait_a_moment, ROUNDS_BEFORE_GIVING_UP);
        });
        self.running = Some((stop_asked, pass));
    }

    fn finish(&mut self) {
        if let Some((stop_asked, pass)) = self.running.take() {
            drop(stop_asked);
            let _ = pass.join();
        }
    }
}

impl Drop for ReopenRunner {
    fn drop(&mut self) {
        self.finish();
    }
}
