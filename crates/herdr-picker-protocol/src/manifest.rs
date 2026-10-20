use herdr_picker_domain::{JumpTarget, Stroke};
use serde::Serialize;

use crate::popup::{CONFIGURED_PICKER_PANE, LAUNCHER_PANE, WORKSPACE_PICKER_PANE};

pub const PLUGIN_ID: &str = "herdr-picker";
const EXECUTABLE_FROM_PLUGIN_ROOT: &str = "./target/release/herdr-picker";
const PICKER_ACTION: &str = "pick";

#[derive(Serialize)]
struct Manifest<'a> {
    id: &'static str,
    name: &'static str,
    version: &'a str,
    min_herdr_version: &'static str,
    description: &'static str,
    platforms: [&'static str; 2],
    build: [BuildStep; 1],
    actions: Vec<Action>,
    events: [Event; 3],
    panes: [Pane; 3],
}

#[derive(Serialize)]
struct BuildStep {
    command: Vec<String>,
}

#[derive(Serialize)]
struct Action {
    id: String,
    title: String,
    command: Vec<String>,
}

#[derive(Serialize)]
struct Event {
    on: &'static str,
    command: Vec<String>,
}

#[derive(Serialize)]
struct Pane {
    id: &'static str,
    title: &'static str,
    placement: &'static str,
    command: Vec<String>,
}

fn herdr_picker(arguments: &[&str]) -> Vec<String> {
    std::iter::once(EXECUTABLE_FROM_PLUGIN_ROOT)
        .chain(arguments.iter().copied())
        .map(str::to_string)
        .collect()
}

fn action(id: &str, title: &str, arguments: &[&str]) -> Action {
    Action {
        id: id.to_string(),
        title: title.to_string(),
        command: herdr_picker(arguments),
    }
}

fn popup(id: &'static str, title: &'static str, subcommand: &str) -> Pane {
    Pane {
        id,
        title,
        placement: "popup",
        command: herdr_picker(&[subcommand]),
    }
}

// `bases` are the keys the launcher reads a second key after, such as ctrl-t; each gets an action.
pub fn render_manifest(
    binary_version: &str,
    targets: &[JumpTarget],
    bases: &[Stroke],
) -> Result<String, toml::ser::Error> {
    let mut actions: Vec<Action> = targets
        .iter()
        .map(|target| {
            action(
                &target.action_id,
                &format!("Jump to the {} workspace", target.label),
                &["jump", &target.label, &target.directory],
            )
        })
        .collect();
    actions.push(action(
        "last_workspace",
        "Last workspace (most-recently-used toggle)",
        &["last-workspace"],
    ));
    actions.push(action(PICKER_ACTION, "Pick a workspace", &["popup"]));
    actions.push(action(
        "bookmark",
        "Bookmark the focused pane's folder",
        &["run", "bookmark"],
    ));
    actions.extend(bases.iter().map(|base| {
        let name = base.name();
        action(
            &format!("launch_{}", name.replace('-', "_")),
            &format!("Pickers on {name}, over the focused pane"),
            &["launch", &name],
        )
    }));
    let event = |on, subcommand| Event {
        on,
        command: herdr_picker(&[subcommand]),
    };
    let manifest = Manifest {
        id: PLUGIN_ID,
        name: "Picker",
        version: binary_version,
        min_herdr_version: "0.7.4",
        description: "Pickers for files, git, agents, workspaces and more, opened from shell keys or a herdr popup.",
        platforms: ["macos", "linux"],
        build: [BuildStep {
            command: ["cargo", "build", "--release", "--locked"]
                .map(str::to_string)
                .to_vec(),
        }],
        actions,
        events: [
            event("workspace.focused", "record"),
            event("pane.focused", "record"),
            event("pane.agent_status_changed", "agent-changed"),
        ],
        panes: [
            popup(WORKSPACE_PICKER_PANE, "Workspace picker", "pick"),
            popup(CONFIGURED_PICKER_PANE, "Picker", "run-in-popup"),
            popup(LAUNCHER_PANE, "Picker launcher", "launcher"),
        ],
    };
    toml::to_string(&manifest)
}

#[cfg(test)]
mod tests;
