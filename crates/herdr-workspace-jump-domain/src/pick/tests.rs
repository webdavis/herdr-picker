use super::*;
use crate::{ReadlineField, Workspace, WorkspaceDeclaration, jump_targets};

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

fn open(labels: &[&str]) -> Vec<Workspace> {
    labels
        .iter()
        .map(|label| Workspace {
            workspace_id: format!("w_{label}"),
            label: label.to_string(),
        })
        .collect()
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

#[derive(Default)]
struct RecordingField {
    edits: Vec<Edit>,
    text: String,
}

impl TextField for RecordingField {
    fn apply(&mut self, edit: Edit) {
        self.edits.push(edit);
        if let Edit::Insert(character) = edit {
            self.text.push(character);
        }
    }

    fn text(&self) -> String {
        self.text.clone()
    }

    fn cursor(&self) -> usize {
        self.text.chars().count()
    }
}

fn menu(workspaces: &[JumpTarget]) -> Menu<'_, Contains, ReadlineField> {
    Menu::new(workspaces, &[], None, Contains, ReadlineField::default())
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

fn search(frame: &Frame<'_>) -> Option<(String, usize)> {
    frame
        .search
        .as_ref()
        .map(|search| (search.text.clone(), search.cursor))
}

fn type_text(menu: &mut Menu<'_, impl Matcher, impl TextField>, text: &str) {
    for character in text.chars() {
        assert_eq!(
            menu.apply(Command::Edit(Edit::Insert(character))),
            Pick::Pending
        );
    }
}
