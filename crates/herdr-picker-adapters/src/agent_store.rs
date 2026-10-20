use std::fs::{self, File};
use std::io;
use std::path::Path;
use std::time::{Duration, SystemTime};

use herdr_picker_domain::{Agent, AgentStatus, Queued, Workspace};
use herdr_picker_protocol::WireTab;
use serde::{Deserialize, Serialize};

use crate::private::{create_private, write_private};

// What herdr runs, saved by the status hook so the agents picker opens without asking herdr.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AgentSnapshot {
    pub agents: Vec<Agent>,
    pub workspaces: Vec<Workspace>,
    pub tabs: Vec<WireTab>,
}

#[derive(Serialize, Deserialize)]
struct SavedSnapshot {
    agents: Vec<SavedAgent>,
    workspaces: Vec<(String, String)>,
    tabs: Vec<(String, String, String)>,
}

#[derive(Serialize, Deserialize)]
struct SavedAgent {
    pane_id: String,
    workspace_id: String,
    tab_id: String,
    kind: String,
    title: String,
    cwd: String,
    status: String,
    since: Option<u64>,
}

#[derive(Serialize, Deserialize)]
struct SavedPrompt {
    pane_id: String,
    text: String,
}

pub fn write_snapshot(path: &Path, snapshot: &AgentSnapshot) -> io::Result<()> {
    let saved = SavedSnapshot {
        agents: snapshot.agents.iter().map(saved_agent).collect(),
        workspaces: (snapshot.workspaces.iter())
            .map(|space| (space.workspace_id.clone(), space.label.clone()))
            .collect(),
        tabs: (snapshot.tabs.iter())
            .map(|tab| {
                (
                    tab.tab_id.clone(),
                    tab.workspace_id.clone(),
                    tab.label.clone(),
                )
            })
            .collect(),
    };
    write_private(path, &serde_json::to_vec(&saved)?)
}

// The saved snapshot while it is younger than `fresh_for`; a missing, old or unreadable one is none.
pub fn read_snapshot(path: &Path, fresh_for: Duration, now: SystemTime) -> Option<AgentSnapshot> {
    let written = fs::metadata(path).and_then(|meta| meta.modified()).ok()?;
    if now.duration_since(written).unwrap_or_default() > fresh_for {
        return None;
    }
    let saved: SavedSnapshot = serde_json::from_slice(&fs::read(path).ok()?).ok()?;
    Some(AgentSnapshot {
        agents: saved.agents.into_iter().map(agent).collect(),
        workspaces: (saved.workspaces.into_iter())
            .map(|(workspace_id, label)| Workspace {
                workspace_id,
                label,
            })
            .collect(),
        tabs: (saved.tabs.into_iter())
            .map(|(tab_id, workspace_id, label)| WireTab {
                tab_id,
                workspace_id,
                label,
            })
            .collect(),
    })
}

// Held from reading the queue to sending from it, so two hooks never take the same prompt or write
// back a list the other has changed; it lets go when dropped.
pub fn lock_queue(path: &Path) -> io::Result<File> {
    let lock = create_private(&path.with_extension("lock"))?;
    lock.lock()?;
    Ok(lock)
}

pub fn write_queue(path: &Path, queue: &[Queued]) -> io::Result<()> {
    let saved: Vec<SavedPrompt> = (queue.iter())
        .map(|queued| SavedPrompt {
            pane_id: queued.pane_id.clone(),
            text: queued.text.clone(),
        })
        .collect();
    write_private(path, &serde_json::to_vec(&saved)?)
}

pub fn read_queue(path: &Path) -> Vec<Queued> {
    let saved: Vec<SavedPrompt> = (fs::read(path).ok())
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default();
    (saved.into_iter())
        .map(|prompt| Queued {
            pane_id: prompt.pane_id,
            text: prompt.text,
        })
        .collect()
}

fn saved_agent(agent: &Agent) -> SavedAgent {
    SavedAgent {
        pane_id: agent.pane_id.clone(),
        workspace_id: agent.workspace_id.clone(),
        tab_id: agent.tab_id.clone(),
        kind: agent.kind.clone(),
        title: agent.title.clone(),
        cwd: agent.cwd.clone(),
        status: agent.status.word().to_string(),
        since: agent.since,
    }
}

fn agent(saved: SavedAgent) -> Agent {
    Agent {
        pane_id: saved.pane_id,
        workspace_id: saved.workspace_id,
        tab_id: saved.tab_id,
        kind: saved.kind,
        title: saved.title,
        cwd: saved.cwd,
        status: AgentStatus::parse(&saved.status),
        since: saved.since,
    }
}

#[cfg(test)]
mod tests;
