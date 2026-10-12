use std::collections::VecDeque;

use herdr_workspace_jump_domain::{Fuzzy, WorkspaceDeclaration, jump_targets};

use super::*;

mod failures;
mod picking;

fn workspaces() -> Vec<JumpTarget> {
    let declared = ["netpulse", "plantpulse", "essential", "dotfiles", "fitness"].map(|label| {
        WorkspaceDeclaration {
            label: label.to_string(),
            directory: format!("/opt/{label}"),
            key: None,
        }
    });
    jump_targets(declared).expect("well formed workspaces")
}

fn menu(targets: &[JumpTarget]) -> Menu<'_, Fuzzy> {
    Menu::new(targets, None, Fuzzy)
}

struct Keys(VecDeque<PickKey>);

impl Keys {
    fn typing(keys: &[PickKey]) -> Self {
        Self(keys.iter().copied().collect())
    }
}

impl KeySource for Keys {
    fn next_key(&mut self) -> io::Result<PickKey> {
        self.0
            .pop_front()
            .ok_or_else(|| io::Error::other("no more keys"))
    }
}

#[derive(Default)]
struct Notes {
    frames: Vec<String>,
    failure: Option<&'static str>,
}

impl Screen for Notes {
    fn show(&mut self, frame: &Frame<'_>) -> io::Result<()> {
        if let Some(failure) = self.failure {
            return Err(io::Error::other(failure));
        }
        let mut line: Vec<String> = frame
            .search
            .iter()
            .map(|search| format!("/{}", search.text))
            .collect();
        line.extend(frame.rows.iter().map(|row| {
            let bar = if row.selected { ">" } else { "" };
            format!("{bar}{}", row.target.label)
        }));
        self.frames.push(line.join(" "));
        Ok(())
    }
}

fn typed(text: &str) -> Vec<PickKey> {
    text.chars().map(PickKey::Character).collect()
}
