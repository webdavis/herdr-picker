mod resume;

use pwe_domain::{Pane, ProgramName, RecordedPane};
use serde::{Deserialize, Serialize};

pub use resume::{NoResumeLine, resume_line};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct StoredPane {
    program: String,
    session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    folder: Option<String>,
    rex_session: String,
    tab: String,
    block: String,
}

pub fn agent_panes_json(panes: &[RecordedPane]) -> String {
    let stored: Vec<StoredPane> = panes.iter().map(stored).collect();
    serde_json::to_string(&stored).unwrap_or_else(|_| "[]".to_string())
}

pub fn parse_agent_panes(text: &str) -> Vec<RecordedPane> {
    serde_json::from_str::<Vec<StoredPane>>(text)
        .unwrap_or_default()
        .into_iter()
        .filter_map(recorded)
        .collect()
}

fn stored(recorded: &RecordedPane) -> StoredPane {
    StoredPane {
        program: recorded.program.word().to_string(),
        session_id: recorded.session_id.clone(),
        folder: recorded.folder.clone(),
        rex_session: recorded.pane.rex_session.clone(),
        tab: recorded.pane.tab.clone(),
        block: recorded.pane.block.clone(),
    }
}

fn recorded(stored: StoredPane) -> Option<RecordedPane> {
    Some(RecordedPane {
        program: ProgramName::parse(&stored.program)?,
        session_id: stored.session_id,
        folder: stored.folder,
        pane: Pane {
            rex_session: stored.rex_session,
            tab: stored.tab,
            block: stored.block,
        },
    })
}

#[cfg(test)]
mod tests;
