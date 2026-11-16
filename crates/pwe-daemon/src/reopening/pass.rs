use pwe_domain::RecordedPane;
use pwe_protocol::{NoResumeLine, resume_line};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PaneNow {
    ShellWaits { folder: Option<String> },
    RunsAProgram,
    NotReady,
}

pub(crate) trait RestoredPanes {
    fn pane_now(&mut self, block: &str) -> PaneNow;
    fn type_line(&mut self, block: &str, line: &str);
}

pub(crate) fn reopen_recorded(
    records: Vec<RecordedPane>,
    panes: &mut dyn RestoredPanes,
    mut wait_a_moment: impl FnMut() -> bool,
    rounds: usize,
) {
    let mut waiting = resumable_once_each(records);
    for _ in 0..rounds {
        waiting.retain(|(record, line)| match panes.pane_now(&record.pane.block) {
            PaneNow::ShellWaits { folder } if folder.is_some() && folder == record.folder => {
                panes.type_line(&record.pane.block, line);
                false
            }
            PaneNow::ShellWaits { .. } => {
                eprintln!(
                    "pwe: does not reopen the agent in {}: its shell is not in the recorded folder",
                    record.pane.block
                );
                false
            }
            PaneNow::RunsAProgram => false,
            PaneNow::NotReady => true,
        });
        if waiting.is_empty() || !wait_a_moment() {
            return;
        }
    }
}

fn resumable_once_each(records: Vec<RecordedPane>) -> Vec<(RecordedPane, String)> {
    let mut resumable: Vec<(RecordedPane, String)> = Vec::new();
    for record in records {
        let already = resumable.iter().any(|(earlier, _)| {
            earlier.program == record.program && earlier.session_id == record.session_id
        });
        match (already, resume_line(&record)) {
            (true, _) | (false, Err(NoResumeLine::ProgramHasNone)) => {}
            (false, Ok(line)) => resumable.push((record, line)),
            (false, Err(refusal)) => eprintln!(
                "pwe: does not reopen the agent in {}: {refusal}",
                record.pane.block
            ),
        }
    }
    resumable
}

#[cfg(test)]
mod tests;
