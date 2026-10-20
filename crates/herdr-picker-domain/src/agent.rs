// herdr's word for what an agent is doing, drawn with herdr's own glyphs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    Blocked,
    Working,
    Done,
    Idle,
    Unknown,
}

impl AgentStatus {
    pub fn parse(word: &str) -> Self {
        match word {
            "blocked" => Self::Blocked,
            "working" => Self::Working,
            "done" => Self::Done,
            "idle" => Self::Idle,
            _ => Self::Unknown,
        }
    }

    pub fn word(self) -> &'static str {
        match self {
            Self::Blocked => "blocked",
            Self::Working => "working",
            Self::Done => "done",
            Self::Idle => "idle",
            Self::Unknown => "unknown",
        }
    }

    pub fn glyph(self) -> &'static str {
        match self {
            Self::Blocked => "◉",
            Self::Working => "◐",
            Self::Done => "●",
            Self::Idle => "✓",
            Self::Unknown => "·",
        }
    }

    // An agent that has finished its turn takes the next queued prompt.
    pub fn settled(self) -> bool {
        matches!(self, Self::Done | Self::Idle)
    }

    fn urgency(self) -> u8 {
        match self {
            Self::Blocked => 0,
            Self::Done => 1,
            _ => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Agent {
    pub pane_id: String,
    pub workspace_id: String,
    pub tab_id: String,
    pub kind: String,
    pub title: String,
    pub cwd: String,
    pub status: AgentStatus,
    // When the status hook last saw its status change, in seconds since the epoch.
    pub since: Option<u64>,
}

// Needs you, then just finished, then the panes visited most recently; the current tab's agents
// go last, so the first row is always somewhere else to go.
pub fn ordered(agents: &[Agent], recent_panes: &[&str], current_tab: Option<&str>) -> Vec<Agent> {
    let mut ranked: Vec<&Agent> = agents.iter().collect();
    ranked.sort_by_key(|agent| {
        let visited = recent_panes
            .iter()
            .position(|pane| *pane == agent.pane_id)
            .unwrap_or(usize::MAX);
        let here = current_tab == Some(agent.tab_id.as_str());
        (here, agent.status.urgency(), visited)
    });
    ranked.into_iter().cloned().collect()
}

// The status hook saw `changed` change now; an agent whose status is as it was keeps its clock,
// and one that changed unseen has none.
pub fn carry_since(agents: Vec<Agent>, before: &[Agent], changed: &str, now: u64) -> Vec<Agent> {
    agents
        .into_iter()
        .map(|agent| {
            let kept = before
                .iter()
                .find(|old| old.pane_id == agent.pane_id && old.status == agent.status)
                .and_then(|old| old.since);
            let since = if agent.pane_id == changed {
                Some(now)
            } else {
                kept
            };
            Agent { since, ..agent }
        })
        .collect()
}

// How long ago, in the largest whole unit: 45s, 3m, 2h, 4d.
pub fn age(seconds: u64) -> String {
    match seconds {
        0..60 => format!("{seconds}s"),
        60..3600 => format!("{}m", seconds / 60),
        3600..86400 => format!("{}h", seconds / 3600),
        _ => format!("{}d", seconds / 86400),
    }
}

#[cfg(test)]
mod tests;
