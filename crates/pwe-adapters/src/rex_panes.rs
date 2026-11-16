use std::path::PathBuf;
use std::process::Command;

use pwe_application::JumpError;
use pwe_protocol::{PaneInfo, rex_pane};

use crate::response::jump_error;

const NEVER_START_A_SERVER: &str = "--autostart=false";

mod host;
mod places;
mod statuses;

pub struct RexPanes {
    binary: PathBuf,
    session: Option<String>,
}

impl RexPanes {
    pub fn at(binary: PathBuf, session: Option<String>) -> Self {
        Self { binary, session }
    }

    pub fn pane(&mut self, target: &str) -> Result<PaneInfo, JumpError> {
        let status = self.run(&["block", "call", "-b", target, "program_status"])?;
        let process = self.run(&["block", "call", "-b", target, "process"])?;
        rex_pane(target, &status, &process).map_err(jump_error)
    }

    pub fn screen(&mut self, target: &str) -> Result<String, JumpError> {
        self.run(&["capture", "-b", target, "--trim"])
    }

    pub(crate) fn run(&self, arguments: &[&str]) -> Result<String, JumpError> {
        let output = Command::new(&self.binary)
            .arg(NEVER_START_A_SERVER)
            .args(arguments)
            .output()
            .map_err(|failure| {
                JumpError::Unreachable(format!("{}: {failure}", self.binary.display()))
            })?;
        if !output.status.success() {
            return Err(JumpError::Transport(format!(
                "rex {} exited {}: {}",
                arguments.join(" "),
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}

#[cfg(test)]
mod tests;
