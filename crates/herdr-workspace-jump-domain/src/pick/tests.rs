use super::*;
use crate::{WorkspaceDeclaration, jump_targets};

mod browsing;

fn targets(declarations: Vec<(&str, Option<&str>)>) -> Vec<JumpTarget> {
    let declared = declarations
        .into_iter()
        .map(|(label, key)| WorkspaceDeclaration {
            label: label.to_string(),
            directory: format!("/opt/{label}"),
            key: key.map(str::to_string),
        })
        .collect::<Vec<_>>();
    jump_targets(declared).expect("well formed workspaces")
}

fn three_workspaces() -> Vec<JumpTarget> {
    targets(vec![
        ("Ivy", None),
        ("homelab", None),
        ("dotfiles", Some("f")),
    ])
}

fn shown(frame: &Frame<'_>) -> Vec<String> {
    frame
        .rows
        .iter()
        .map(|row| {
            let bar = if row.selected { ">" } else { "" };
            format!("{bar}{}", row.target.label)
        })
        .collect()
}
