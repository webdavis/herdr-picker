use std::fs;
use std::path::{Path, PathBuf};

use pwe_application::{AgentProgram, AgentReadError, OpenFiles};
use pwe_domain::ProgramReport;
use pwe_protocol::codex_session;

use super::files::{file_name_of, modified_seconds};

pub struct CodexSessions<F> {
    codex_folder: PathBuf,
    open_files: F,
}

impl<F: OpenFiles> CodexSessions<F> {
    pub fn on(codex_folder: &Path, open_files: F) -> Self {
        Self {
            codex_folder: codex_folder.to_path_buf(),
            open_files,
        }
    }

    fn held_rollout_files(&self) -> Vec<PathBuf> {
        let sessions = self.codex_folder.join("sessions");
        let resolved = fs::canonicalize(&sessions).unwrap_or_else(|_| sessions.clone());
        let mut held: Vec<PathBuf> = self
            .open_files
            .opened_by("codex")
            .into_iter()
            .map(|file| file.path)
            .filter(|path| path.starts_with(&sessions) || path.starts_with(&resolved))
            .filter(|path| is_a_rollout_file(path))
            .collect();
        held.sort();
        held.dedup();
        held
    }
}

impl<F: OpenFiles> AgentProgram for CodexSessions<F> {
    fn sessions(&mut self) -> Result<Vec<ProgramReport>, AgentReadError> {
        Ok(self
            .held_rollout_files()
            .iter()
            .filter_map(|file| {
                let report = codex_session(&fs::read_to_string(file).ok()?)?;
                Some(ProgramReport {
                    state_since: modified_seconds(file),
                    ..report
                })
            })
            .collect())
    }
}

fn is_a_rollout_file(path: &Path) -> bool {
    file_name_of(path).is_some_and(|name| name.starts_with("rollout-") && name.ends_with(".jsonl"))
}

#[cfg(test)]
mod tests;
