#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pub workspace_id: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Jump {
    Focused(String),
    Created,
}

pub fn decide(workspaces: &[Workspace], label: &str) -> Jump {
    match workspaces.iter().find(|workspace| workspace.label == label) {
        Some(workspace) => Jump::Focused(workspace.workspace_id.clone()),
        None => Jump::Created,
    }
}

#[cfg(test)]
mod tests;
