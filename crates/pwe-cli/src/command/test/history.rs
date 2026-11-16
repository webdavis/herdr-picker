use std::env;
use std::path::{Path, PathBuf};

use pwe_adapters::pwe_state_folder;
use pwe_adapters::test_engine::{read_stored_run, run_folder, stored_run_ids_newest_first};
use pwe_domain::test_engine::{RunFinished, TestEvent};
use pwe_protocol::test_engine::read_event_line;

pub(super) struct StoredRun {
    pub(super) lines: String,
    pub(super) events: Vec<TestEvent>,
}

impl StoredRun {
    fn read(lines: String) -> Self {
        let events = lines.lines().filter_map(read_event_line).collect();
        StoredRun { lines, events }
    }

    pub(super) fn finished(&self) -> Option<&RunFinished> {
        self.events.iter().rev().find_map(|event| match event {
            TestEvent::RunFinished(finished) => Some(finished),
            _ => None,
        })
    }
}

pub(super) fn runs_of(repo_root: &Path) -> PathBuf {
    run_folder(
        &pwe_state_folder(
            env::var("XDG_STATE_HOME").ok().as_deref(),
            env::var("HOME").ok().as_deref(),
        ),
        repo_root,
    )
}

pub(super) fn stored_run(runs: &Path, run_id: &str) -> Option<StoredRun> {
    read_stored_run(runs, run_id).map(StoredRun::read)
}

pub(super) fn last_finished_run(runs: &Path) -> Option<StoredRun> {
    stored_run_ids_newest_first(runs)
        .iter()
        .filter_map(|run_id| stored_run(runs, run_id))
        .find(|run| run.finished().is_some())
}
