use herdr_picker_domain::JumpTarget;
use serde::Serialize;

use crate::popup::{CONFIGURED_PICKER_PANE, WORKSPACE_PICKER_PANE};

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
    events: [Event; 1],
    panes: [Pane; 2],
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

pub fn render_manifest(
    binary_version: &str,
    targets: &[JumpTarget],
) -> Result<String, toml::ser::Error> {
    let mut actions: Vec<Action> = targets
        .iter()
        .map(|target| Action {
            id: target.action_id.clone(),
            title: format!("Jump to the {} workspace", target.label),
            command: vec![
                EXECUTABLE_FROM_PLUGIN_ROOT.to_string(),
                "jump".to_string(),
                target.label.clone(),
                target.directory.clone(),
            ],
        })
        .collect();
    actions.push(Action {
        id: "last_workspace".to_string(),
        title: "Last workspace (most-recently-used toggle)".to_string(),
        command: vec![
            EXECUTABLE_FROM_PLUGIN_ROOT.to_string(),
            "last-workspace".to_string(),
        ],
    });
    actions.push(Action {
        id: PICKER_ACTION.to_string(),
        title: "Pick a workspace".to_string(),
        command: vec![EXECUTABLE_FROM_PLUGIN_ROOT.to_string(), "popup".to_string()],
    });
    actions.push(Action {
        id: "bookmark".to_string(),
        title: "Bookmark the focused pane's folder".to_string(),
        command: vec![
            EXECUTABLE_FROM_PLUGIN_ROOT.to_string(),
            "run".to_string(),
            "bookmark".to_string(),
        ],
    });
    let manifest = Manifest {
        id: PLUGIN_ID,
        name: "Picker",
        version: binary_version,
        min_herdr_version: "0.7.4",
        description: "Workspace navigation herdr has no built-in for: create-or-focus by label, a most-recently-used toggle, and a workspace picker.",
        platforms: ["macos", "linux"],
        build: [BuildStep {
            command: vec![
                "cargo".to_string(),
                "build".to_string(),
                "--release".to_string(),
                "--locked".to_string(),
            ],
        }],
        actions,
        events: [Event {
            on: "workspace.focused",
            command: vec![
                EXECUTABLE_FROM_PLUGIN_ROOT.to_string(),
                "record".to_string(),
            ],
        }],
        panes: [
            Pane {
                id: WORKSPACE_PICKER_PANE,
                title: "Workspace picker",
                placement: "popup",
                command: vec![EXECUTABLE_FROM_PLUGIN_ROOT.to_string(), "pick".to_string()],
            },
            Pane {
                id: CONFIGURED_PICKER_PANE,
                title: "Picker",
                placement: "popup",
                command: vec![
                    EXECUTABLE_FROM_PLUGIN_ROOT.to_string(),
                    "run-in-popup".to_string(),
                ],
            },
        ],
    };
    toml::to_string(&manifest)
}

#[cfg(test)]
mod tests;
