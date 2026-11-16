use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use pwe_application::{AgentProgram, AgentReadError, OpenFile, OpenFiles};
use pwe_domain::{ProgramName, ProgramReport};
use pwe_protocol::{omp_held_session_ids, omp_session};

use super::files::{entries_of, file_name_of, modified_seconds};
use super::printed::{printed_by, unreadable};
use crate::program::on_path;

const LOCK_ENDING: &str = ".lock";

pub struct OmpSessions<F> {
    omp: Option<PathBuf>,
    omp_folder: PathBuf,
    open_files: F,
}

impl<F: OpenFiles> OmpSessions<F> {
    pub fn on(path_variable: Option<&OsStr>, omp_folder: &Path, open_files: F) -> Self {
        Self {
            omp: on_path("omp", path_variable),
            omp_folder: omp_folder.to_path_buf(),
            open_files,
        }
    }

    fn held_session_ids(&self, omp: &Path) -> Result<Vec<String>, AgentReadError> {
        let locked = self.locked_session_ids();
        if locked.is_empty() {
            return Ok(Vec::new());
        }
        let mut arguments = vec!["session", "lease", "--json", "--"];
        arguments.extend(locked.iter().map(String::as_str));
        let printed = printed_by(omp, &arguments).map_err(refusal)?;
        omp_held_session_ids(&printed).map_err(|error| refusal(unreadable(error)))
    }

    fn locked_session_ids(&self) -> Vec<String> {
        entries_of(&self.omp_folder.join("run/session-owners"))
            .iter()
            .filter_map(|lock| file_name_of(lock)?.strip_suffix(LOCK_ENDING))
            .map(str::to_string)
            .collect()
    }

    fn session(&self, id: &str, holders: &[OpenFile]) -> Option<ProgramReport> {
        let file = self.session_file(id)?;
        let report = omp_session(&fs::read_to_string(&file).ok()?)?;
        let lock = format!("{id}{LOCK_ENDING}");
        Some(ProgramReport {
            pid: holders
                .iter()
                .find(|held| file_name_of(&held.path) == Some(lock.as_str()))
                .map(|held| held.pid),
            state_since: modified_seconds(&file),
            ..report
        })
    }

    fn session_file(&self, id: &str) -> Option<PathBuf> {
        let ending = format!("_{id}.jsonl");
        entries_of(&self.omp_folder.join("agent/sessions"))
            .iter()
            .flat_map(|folder| entries_of(folder))
            .find(|file| file_name_of(file).is_some_and(|name| name.ends_with(&ending)))
    }
}

impl<F: OpenFiles> AgentProgram for OmpSessions<F> {
    fn sessions(&mut self) -> Result<Vec<ProgramReport>, AgentReadError> {
        let Some(omp) = &self.omp else {
            return Ok(Vec::new());
        };
        let held = self.held_session_ids(omp)?;
        if held.is_empty() {
            return Ok(Vec::new());
        }
        let holders = self.open_files.opened_by("omp");
        Ok(held
            .iter()
            .filter_map(|id| self.session(id, &holders))
            .collect())
    }
}

fn refusal(detail: String) -> AgentReadError {
    AgentReadError {
        program: ProgramName::Omp.word().to_string(),
        detail,
    }
}

#[cfg(test)]
mod tests;
