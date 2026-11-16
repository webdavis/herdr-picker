use pwe_domain::{Jump, Workspace, decide};

use crate::JumpError;

pub trait WorkspaceDirectory {
    fn list(&mut self) -> Result<Vec<Workspace>, JumpError>;
    fn focus(&mut self, workspace_id: &str) -> Result<(), JumpError>;
    fn create(&mut self, label: &str, cwd: &str) -> Result<(), JumpError>;
}

pub fn jump(
    directory: &mut impl WorkspaceDirectory,
    label: &str,
    cwd: &str,
) -> Result<Jump, JumpError> {
    let outcome = decide(&directory.list()?, label);
    match &outcome {
        Jump::Focused(workspace_id) => directory.focus(workspace_id)?,
        Jump::Created => directory.create(label, cwd)?,
    }
    Ok(outcome)
}

#[cfg(test)]
mod tests;
