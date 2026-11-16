#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaneState {
    Running { pid: u32 },
    Finished { instruction: Option<String> },
    Failed { reason: String },
}

pub fn encode_pane_state(state: &PaneState) -> String {
    match state {
        PaneState::Running { pid } => format!("running\n{pid}"),
        PaneState::Finished { instruction } => {
            format!("finished\n{}", instruction.as_deref().unwrap_or_default())
        }
        PaneState::Failed { reason } => format!("failed\n{reason}"),
    }
}

pub fn parse_pane_state(body: &str) -> Option<PaneState> {
    let (word, rest) = body.split_once('\n')?;
    match word {
        "running" => rest.parse().ok().map(|pid| PaneState::Running { pid }),
        "finished" => Some(PaneState::Finished {
            instruction: Some(rest.to_string()).filter(|instruction| !instruction.is_empty()),
        }),
        "failed" => Some(PaneState::Failed {
            reason: rest.to_string(),
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
