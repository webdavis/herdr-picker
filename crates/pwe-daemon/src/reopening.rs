mod pass;
mod rex_panes;
mod runner;

use std::path::PathBuf;

use pwe_adapters::{read_agent_panes, save_agent_panes};
use pwe_domain::{AgentSession, RecordedPane, recorded_panes};
use pwe_protocol::{RexLink, resume_line};

pub(crate) use runner::ReopenRunner;

pub(crate) type Reopen = Box<dyn FnMut(Vec<RecordedPane>) + Send>;

pub(crate) struct Reopening {
    file: PathBuf,
    written: Option<Vec<RecordedPane>>,
    rex_went_away: bool,
    reopen: Reopen,
}

impl Reopening {
    pub(crate) fn new(file: PathBuf, reopen: Reopen) -> Self {
        Self {
            file,
            written: None,
            rex_went_away: false,
            reopen,
        }
    }

    pub(crate) fn record(&mut self, sessions: &[AgentSession]) {
        let panes: Vec<RecordedPane> = recorded_panes(sessions)
            .into_iter()
            .filter(|recorded| resume_line(recorded).is_ok())
            .collect();
        if self.written.as_ref() == Some(&panes) {
            return;
        }
        match save_agent_panes(&self.file, &panes) {
            Ok(()) => self.written = Some(panes),
            Err(failure) => eprintln!(
                "pwe: could not record the agent panes in {}: {failure}",
                self.file.display()
            ),
        }
    }

    pub(crate) fn hear_link(&mut self, link: RexLink) {
        match link {
            RexLink::Reconnecting | RexLink::Absent => self.rex_went_away = true,
            RexLink::Connected if self.rex_went_away => {
                self.rex_went_away = false;
                match read_agent_panes(&self.file) {
                    Ok(panes) => (self.reopen)(panes),
                    Err(failure) => eprintln!("pwe: reopens no agent: {failure}"),
                }
            }
            RexLink::Connected => {}
        }
    }
}
