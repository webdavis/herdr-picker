use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use herdr_picker_domain::{Action, NewPane, PickerRow, Preview, age};

use super::instructions::claude_project;
use crate::builtin::Places;
use crate::builtin::git::Repo;
use crate::builtin::page::{notice, words};
use crate::builtin::paint::dim;
use transcript::{claude_prompt, codex_prompt, first_prompt};

mod transcript;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Session {
    pub(super) resume: String,
    pub(super) prompt: String,
    pub(super) file: PathBuf,
    pub(super) at: u64,
}

pub(super) fn sessions(places: &Places) -> Vec<PickerRow> {
    let root = Repo::at(&places.cwd).map_or_else(|_| places.cwd.clone(), |repo| repo.root);
    let found = places
        .home
        .as_deref()
        .map(|home| past_sessions(&root, home))
        .unwrap_or_default();
    if found.is_empty() {
        return vec![notice(
            "No past sessions here",
            "Claude and Codex sessions started in this repository show up here.",
        )];
    }
    let now = seconds(SystemTime::now());
    found
        .into_iter()
        .map(|session| {
            let first_line = session
                .prompt
                .lines()
                .next()
                .unwrap_or_default()
                .to_string();
            let tool = session.resume.split(' ').next().unwrap_or_default();
            let when = format!("{tool} · {} ago", age(now.saturating_sub(session.at)));
            PickerRow {
                label: format!("{first_line}  {}", dim(&when)),
                preview: Some(Preview::Text(format!(
                    "{}\n\n{}\n{}",
                    session.prompt,
                    session.resume,
                    session.file.display()
                ))),
                enter: Action::InPane {
                    pane: NewPane::Right,
                    action: Box::new(Action::Run(session.resume)),
                },
                keys: Vec::new(),
                group: None,
            }
        })
        .collect()
}

// Claude and Codex sessions started in the repository, newest first, each with its first prompt.
pub(super) fn past_sessions(root: &Path, home: &Path) -> Vec<Session> {
    let project = claude_project(home, root);
    let prefix = project
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let claude_folders = listed(&home.join(".claude/projects"))
        .into_iter()
        .filter(|folder| {
            let name = folder.file_name().unwrap_or_default().to_string_lossy();
            name == prefix || name.starts_with(&format!("{prefix}-"))
        });
    let claude = claude_folders
        .flat_map(|folder| listed(&folder))
        .filter_map(|file| {
            let (cwd, prompt) = first_prompt(&file, claude_prompt)?;
            let id = file.file_stem()?.to_string_lossy().into_owned();
            Path::new(&cwd)
                .starts_with(root)
                .then(|| session(words(&["claude", "--resume", &id]), prompt, file))
        });
    let days = listed(&home.join(".codex/sessions"))
        .into_iter()
        .flat_map(|year| listed(&year))
        .flat_map(|month| listed(&month));
    let codex = days.flat_map(|day| listed(&day)).filter_map(|file| {
        let (meta, prompt) = first_prompt(&file, codex_prompt)?;
        let (cwd, id) = meta.split_once('\t')?;
        Path::new(cwd)
            .starts_with(root)
            .then(|| session(words(&["codex", "resume", id]), prompt, file))
    });
    let mut found: Vec<Session> = claude.chain(codex).collect();
    found.sort_by(|left, right| right.at.cmp(&left.at).then(left.file.cmp(&right.file)));
    found
}

fn session(resume: String, prompt: String, file: PathBuf) -> Session {
    let at = fs::metadata(&file)
        .and_then(|meta| meta.modified())
        .map_or(0, seconds);
    Session {
        resume,
        prompt,
        file,
        at,
    }
}

fn listed(folder: &Path) -> Vec<PathBuf> {
    fs::read_dir(folder)
        .into_iter()
        .flatten()
        .filter_map(|entry| Some(entry.ok()?.path()))
        .collect()
}

fn seconds(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs())
}

#[cfg(test)]
mod tests;
