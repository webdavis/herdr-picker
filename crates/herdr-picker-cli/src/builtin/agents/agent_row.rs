use std::path::Path;

use herdr_picker_domain::{Action, Agent, AgentStatus, HerdrCall, PickerRow, Preview, age};

use super::branch::branch;
use super::herdr_view::HerdrView;
use crate::builtin::paint::{blue, dim, green, peach, yellow};
use crate::builtin::rows::{in_home, key};

pub(super) fn status_glyph(status: AgentStatus) -> String {
    let glyph = status.glyph();
    match status {
        AgentStatus::Blocked => peach(glyph),
        AgentStatus::Working => yellow(glyph),
        AgentStatus::Done => blue(glyph),
        AgentStatus::Idle => green(glyph),
        AgentStatus::Unknown => dim(glyph),
    }
}

pub(super) fn workspace_label<'a>(view: &'a HerdrView, workspace_id: &'a str) -> &'a str {
    view.snapshot
        .workspaces
        .iter()
        .find(|space| space.workspace_id == workspace_id)
        .map_or(workspace_id, |space| space.label.as_str())
}

pub(super) fn tab_label<'a>(view: &'a HerdrView, tab_id: &'a str) -> &'a str {
    view.snapshot
        .tabs
        .iter()
        .find(|tab| tab.tab_id == tab_id)
        .map_or(tab_id, |tab| tab.label.as_str())
}

// The title, then what a search should also find: the agent, where it is, its folder and branch.
pub(super) fn agent_row(agent: &Agent, view: &HerdrView, home: Option<&Path>) -> PickerRow {
    let tab = tab_label(view, &agent.tab_id);
    let place = format!("{} › {tab}", workspace_label(view, &agent.workspace_id));
    let folder = in_home(Path::new(&agent.cwd), home);
    let branch = branch(Path::new(&agent.cwd)).map(|(name, _)| name);
    let running = agent.since.map(|since| {
        format!(
            "{} {}",
            agent.status.word(),
            age(view.now.saturating_sub(since))
        )
    });
    let details: Vec<String> = [
        Some(agent.kind.clone()),
        Some(place),
        Some(folder),
        branch,
        running,
    ]
    .into_iter()
    .flatten()
    .collect();
    let read = [
        "agent",
        "read",
        &agent.pane_id,
        "--source",
        "recent-unwrapped",
        "--lines",
        "80",
    ];
    PickerRow {
        label: format!(
            "{} {}  {}",
            status_glyph(agent.status),
            agent.title,
            dim(&details.join(" · "))
        ),
        preview: Some(Preview::Command(
            std::iter::once(view.herdr.as_str())
                .chain(read)
                .map(str::to_string)
                .collect(),
        )),
        enter: Action::Herdr(HerdrCall::FocusPane(agent.pane_id.clone())),
        keys: vec![close_tab(&agent.tab_id, tab)],
        group: None,
    }
}

pub(super) fn close_tab(tab_id: &str, label: &str) -> (herdr_picker_domain::Key, Action) {
    (
        key("alt-x"),
        Action::Confirm {
            question: format!("Close the tab {label} and everything running in it?"),
            action: Box::new(Action::Herdr(HerdrCall::CloseTab(tab_id.to_string()))),
        },
    )
}
