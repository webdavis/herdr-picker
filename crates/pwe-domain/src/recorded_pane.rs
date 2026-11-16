use crate::{AgentSession, Pane, ProgramName};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedPane {
    pub program: ProgramName,
    pub session_id: String,
    pub folder: Option<String>,
    pub pane: Pane,
}

pub fn recorded_panes(sessions: &[AgentSession]) -> Vec<RecordedPane> {
    sessions
        .iter()
        .filter_map(|session| {
            Some(RecordedPane {
                program: session.program,
                session_id: session.session_id.clone(),
                folder: session.folder.clone(),
                pane: session.pane.clone()?,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;
