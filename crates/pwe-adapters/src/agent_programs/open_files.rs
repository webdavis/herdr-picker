use std::ffi::OsStr;
use std::path::PathBuf;

use pwe_application::{OpenFile, OpenFiles};
use pwe_protocol::open_files_in_lsof_listing;

use super::printed::printed_by;
use crate::program::on_path;

pub struct LsofOpenFiles {
    lsof: Option<PathBuf>,
}

impl LsofOpenFiles {
    pub fn on(path_variable: Option<&OsStr>) -> Self {
        Self {
            lsof: on_path("lsof", path_variable),
        }
    }
}

impl OpenFiles for LsofOpenFiles {
    fn opened_by(&self, command: &str) -> Vec<OpenFile> {
        let Some(lsof) = &self.lsof else {
            return Vec::new();
        };
        let listing = printed_by(lsof, &["-w", "-F", "pn", "-c", command]).unwrap_or_default();
        open_files_in_lsof_listing(&listing)
            .into_iter()
            .map(|(pid, path)| OpenFile { pid, path })
            .collect()
    }
}

#[cfg(test)]
mod tests;
