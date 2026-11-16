use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use pwe_application::ProcessEnvironment;
use pwe_protocol::{variable_in_environ_file, variable_in_ps_listing};

use super::printed::printed_by;

use crate::program::on_path;

pub struct SystemProcessEnvironment {
    ps: Option<PathBuf>,
    proc_folder: PathBuf,
}

impl SystemProcessEnvironment {
    pub fn on(path_variable: Option<&OsStr>, proc_folder: &Path) -> Self {
        Self {
            ps: on_path("ps", path_variable),
            proc_folder: proc_folder.to_path_buf(),
        }
    }
}

impl ProcessEnvironment for SystemProcessEnvironment {
    fn variable(&self, pid: u32, name: &str) -> Option<String> {
        let environ_file = self.proc_folder.join(pid.to_string()).join("environ");
        if let Ok(contents) = fs::read(environ_file) {
            return variable_in_environ_file(&contents, name);
        }
        let listing = printed_by(self.ps.as_ref()?, &["-E", "-ww", "-p", &pid.to_string()]).ok()?;
        variable_in_ps_listing(&listing, name)
    }
}

#[cfg(test)]
mod tests;
