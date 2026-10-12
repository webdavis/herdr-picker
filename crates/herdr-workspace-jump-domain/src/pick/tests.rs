use super::*;
use crate::{WorkspaceDeclaration, jump_targets};

mod browsing;
mod searching;

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

struct Contains;

impl Matcher for Contains {
    fn rank<'t, T>(&self, query: &str, items: &'t [T], label: impl Fn(&T) -> &str) -> Vec<&'t T> {
        items
            .iter()
            .filter(|item| label(item).contains(query))
            .collect()
    }
}

struct Reversed;

impl Matcher for Reversed {
    fn rank<'t, T>(&self, _: &str, items: &'t [T], _: impl Fn(&T) -> &str) -> Vec<&'t T> {
        items.iter().rev().collect()
    }
}

fn menu(workspaces: &[JumpTarget]) -> Menu<'_, Contains> {
    Menu::new(workspaces, None, Contains)
}

fn shown(frame: &Frame<'_>) -> Vec<String> {
    frame
        .rows
        .iter()
        .map(|row| {
            let bar = if row.selected { ">" } else { "" };
            let current = if row.current { "*" } else { "" };
            format!("{bar}{}{current}", row.target.label)
        })
        .collect()
}

fn search(frame: &Frame<'_>) -> Option<String> {
    frame.search.as_ref().map(|search| search.text.clone())
}

fn type_text(menu: &mut Menu<'_, impl Matcher>, text: &str) {
    for character in text.chars() {
        assert_eq!(menu.decide(PickKey::Character(character)), Pick::Pending);
    }
}
