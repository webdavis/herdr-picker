use std::path::Path;

use herdr_picker_domain::{Action, Agent, AgentStatus, HerdrCall, PickerRow, Preview, ordered};

use super::agent_row::{agent_row, close_tab, status_glyph, tab_label, workspace_label};
use super::branch::branch;
use super::herdr_view::HerdrView;
use crate::builtin::page::notice;
use crate::builtin::paint::{blue, dim};
use crate::builtin::rows::in_home;

pub(super) fn rows(source: &str, view: &HerdrView, home: Option<&Path>) -> Vec<PickerRow> {
    let recent: Vec<&str> = view
        .recent
        .iter()
        .map(|place| place.pane_id.as_str())
        .collect();
    let agents = ordered(&view.snapshot.agents, &recent, view.tab.as_deref());
    let picked = |keep: &dyn Fn(&Agent) -> bool| -> Vec<PickerRow> {
        agents
            .iter()
            .filter(|agent| keep(agent))
            .map(|agent| agent_row(agent, view, home))
            .collect()
    };
    let listed = match source {
        "needs you" => picked(&|agent| agent.status == AgentStatus::Blocked),
        "done" => picked(&|agent| agent.status == AgentStatus::Done),
        "agents" => picked(&|_| true),
        "terminals" => terminals(view),
        "workspaces" => workspaces(view),
        "recent" => recent_places(view, home),
        "worktrees" => {
            picked(&|agent| branch(Path::new(&agent.cwd)).is_some_and(|(_, linked)| linked))
        }
        _ => picked(&|_| true)
            .into_iter()
            .chain(terminals(view))
            .collect(),
    };
    match listed.is_empty() {
        true => vec![notice(
            &format!("Nothing in {source}"),
            "Ctrl-R asks herdr again.",
        )],
        false => listed,
    }
}

// Tabs with no agent in them.
fn terminals(view: &HerdrView) -> Vec<PickerRow> {
    view.snapshot
        .tabs
        .iter()
        .filter(|tab| {
            view.snapshot
                .agents
                .iter()
                .all(|agent| agent.tab_id != tab.tab_id)
        })
        .map(|tab| {
            let workspace = workspace_label(view, &tab.workspace_id);
            PickerRow {
                label: format!("{}  {}", tab.label, dim(workspace)),
                preview: Some(Preview::Text(format!("Tab {} in {workspace}", tab.label))),
                enter: Action::Herdr(HerdrCall::FocusTab(tab.tab_id.clone())),
                keys: vec![close_tab(&tab.tab_id, &tab.label)],
                group: None,
            }
        })
        .collect()
}

fn workspaces(view: &HerdrView) -> Vec<PickerRow> {
    view.snapshot
        .workspaces
        .iter()
        .map(|space| {
            let tabs = view
                .snapshot
                .tabs
                .iter()
                .filter(|tab| tab.workspace_id == space.workspace_id);
            let lines: Vec<String> = tabs
                .map(|tab| {
                    let agents = view
                        .snapshot
                        .agents
                        .iter()
                        .filter(|agent| agent.tab_id == tab.tab_id);
                    let held: Vec<String> = agents
                        .map(|agent| format!("  {} {}", agent.status.glyph(), agent.title))
                        .collect();
                    std::iter::once(tab.label.clone())
                        .chain(held)
                        .collect::<Vec<_>>()
                        .join("\n")
                })
                .collect();
            PickerRow {
                label: blue(&space.label),
                preview: Some(Preview::Text(lines.join("\n"))),
                enter: Action::Herdr(HerdrCall::FocusWorkspace(space.workspace_id.clone())),
                keys: Vec::new(),
                group: None,
            }
        })
        .collect()
}

// The places the record hook logged, newest first, while their tab is still open.
fn recent_places(view: &HerdrView, home: Option<&Path>) -> Vec<PickerRow> {
    view.recent
        .iter()
        .filter(|place| {
            view.snapshot
                .tabs
                .iter()
                .any(|tab| tab.tab_id == place.tab_id)
        })
        .map(|place| {
            let agent = view
                .snapshot
                .agents
                .iter()
                .find(|agent| agent.pane_id == place.pane_id);
            let glyph = agent.map_or_else(|| " ".to_string(), |agent| status_glyph(agent.status));
            let where_ = format!(
                "{} › {}",
                workspace_label(view, &place.workspace_id),
                tab_label(view, &place.tab_id)
            );
            PickerRow {
                label: format!(
                    "{glyph} {}  {}",
                    in_home(Path::new(&place.cwd), home),
                    dim(&where_)
                ),
                preview: Some(Preview::Path(place.cwd.clone())),
                enter: Action::Herdr(HerdrCall::FocusPane(place.pane_id.clone())),
                keys: Vec::new(),
                group: None,
            }
        })
        .collect()
}
