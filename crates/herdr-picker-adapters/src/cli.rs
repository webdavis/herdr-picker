use std::process::Command;

use herdr_picker_application::{JumpError, WorkspaceDirectory};
use herdr_picker_domain::Workspace;
use herdr_picker_protocol as protocol;
use serde_json::{Value, json};

use crate::response;

const BARE_NAME_ON_PATH: &str = "herdr";

pub struct CliWorkspaceDirectory {
    binary: String,
}

impl CliWorkspaceDirectory {
    pub fn using(injected_path: Option<String>) -> Self {
        Self {
            binary: injected_path
                .filter(|path| !path.is_empty())
                .unwrap_or_else(|| BARE_NAME_ON_PATH.to_string()),
        }
    }

    fn run(&self, arguments: &[&str]) -> Result<Value, JumpError> {
        let output = Command::new(&self.binary)
            .args(arguments)
            .output()
            .map_err(|error| JumpError::Unreachable(format!("{}: {error}", self.binary)))?;
        let result = envelope_or_silent_acknowledgment(&String::from_utf8_lossy(&output.stdout));
        if !output.status.success() && result.is_ok() {
            let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(JumpError::Transport(format!(
                "{} {} exited {}: {detail}",
                self.binary,
                arguments.join(" "),
                output.status
            )));
        }
        result
    }
}

fn envelope_or_silent_acknowledgment(printed: &str) -> Result<Value, JumpError> {
    if printed.trim().is_empty() {
        return Ok(json!({}));
    }
    protocol::result_of(printed).map_err(response::jump_error)
}

impl WorkspaceDirectory for CliWorkspaceDirectory {
    fn list(&mut self) -> Result<Vec<Workspace>, JumpError> {
        let result = self.run(&["workspace", "list"])?;
        response::workspaces(&result)
    }

    fn focus(&mut self, workspace_id: &str) -> Result<(), JumpError> {
        self.run(&["workspace", "focus", workspace_id])?;
        Ok(())
    }

    fn create(&mut self, label: &str, cwd: &str) -> Result<(), JumpError> {
        self.run(&[
            "workspace",
            "create",
            "--cwd",
            cwd,
            "--label",
            label,
            "--focus",
        ])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
