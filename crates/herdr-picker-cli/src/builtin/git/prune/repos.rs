use std::collections::BTreeSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs, thread};

use herdr_picker_domain::{Action, PickerRow, Preview};
use herdr_picker_protocol::shell_word;
use serde_json::Value;

use super::super::paint::dim;
use super::super::steps::next;
use super::super::{Repo, output, plural};
use super::dry_run;
use crate::builtin::rows::{RowWriter, in_home, key};
use crate::builtin::{Places, written};
use crate::command::CommandError;

// Every repository with something to prune: this one, and the ones whose worktrees sit in
// worktrunk's worktree folder; each is asked at the same time.
pub(in super::super) fn repos(
    repo: &Repo,
    places: &Places,
    rows: &mut RowWriter<impl Write>,
) -> Result<(), CommandError> {
    let mut found = BTreeSet::from([repo.root.clone()]);
    if let Some(base) = configured_base(places.home.as_deref()) {
        found.extend(
            worktrees_in(&base)
                .iter()
                .filter_map(|tree| main_repo(tree)),
        );
    }
    let answers: Vec<(PathBuf, Vec<Value>)> = thread::scope(|scope| {
        let asking: Vec<_> = found
            .into_iter()
            .map(|root| {
                scope.spawn(move || {
                    let said = dry_run(&root.to_string_lossy(), &["--format=json"]);
                    let items = serde_json::from_str::<Vec<Value>>(&said).unwrap_or_default();
                    (root, items)
                })
            })
            .collect();
        asking
            .into_iter()
            .filter_map(|answer| answer.join().ok())
            .collect()
    });
    let listed: Vec<PickerRow> = answers
        .into_iter()
        .filter(|(_, items)| !items.is_empty())
        .map(|(root, items)| repo_row(&root, &items, places))
        .collect();
    if listed.is_empty() {
        written(rows.header("No repository has anything to prune."))?;
    }
    written(listed.iter().try_for_each(|row| rows.row(row)))
}

fn repo_row(root: &Path, items: &[Value], places: &Places) -> PickerRow {
    let path = root.to_string_lossy().into_owned();
    let text = |item: &Value, name: &str| item[name].as_str().unwrap_or_default().to_string();
    let lines: Vec<String> = items
        .iter()
        .map(|item| {
            format!(
                "{} {} ({} {})",
                text(item, "kind"),
                text(item, "branch"),
                text(item, "reason"),
                text(item, "target")
            )
        })
        .collect();
    let prune = next("prune", &[&path], None, None);
    PickerRow {
        label: format!(
            "{}  {}",
            in_home(root, places.home.as_deref()),
            dim(&format!("{} to prune", plural(items.len(), "item")))
        ),
        preview: Some(Preview::Text(lines.join("\n"))),
        enter: Action::Insert(shell_word(&path)),
        keys: vec![(key("ctrl-enter"), Action::Cd(path)), (key("alt-d"), prune)],
        group: None,
    }
}

fn configured_base(home: Option<&Path>) -> Option<PathBuf> {
    let shown = output(Command::new("wt").args(["config", "show", "--format=json"])).ok()?;
    let shown: Value = serde_json::from_slice(&shown).ok()?;
    worktree_base(shown["user"]["config"]["worktree-path"].as_str()?, home)
}

// The fixed folder a worktree path template starts with; None for one beside each repository.
pub(super) fn worktree_base(template: &str, home: Option<&Path>) -> Option<PathBuf> {
    let fixed = template.split("{{").next()?.trim_end_matches('/');
    match fixed.strip_prefix("~/") {
        Some(inside) => Some(home?.join(inside)),
        None => fixed.starts_with('/').then(|| PathBuf::from(fixed)),
    }
}

// Worktrees a few folders down: each has a .git file pointing at its repository.
fn worktrees_in(base: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut level = vec![base.to_path_buf()];
    for _ in 0..3 {
        let inside = level
            .iter()
            .flat_map(|folder| fs::read_dir(folder).into_iter().flatten().flatten());
        let folders: Vec<PathBuf> = inside
            .map(|entry| entry.path())
            .filter(|path| path.is_dir())
            .collect();
        found.extend(
            folders
                .iter()
                .filter(|folder| folder.join(".git").is_file())
                .cloned(),
        );
        level = folders
            .into_iter()
            .filter(|folder| !folder.join(".git").exists())
            .collect();
    }
    found
}

pub(super) fn main_repo(tree: &Path) -> Option<PathBuf> {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(tree)
        .args(["rev-parse", "--path-format=absolute", "--git-common-dir"]);
    let common = String::from_utf8_lossy(&output(&mut command).ok()?)
        .trim()
        .to_string();
    let common = PathBuf::from(common);
    (common.file_name()? == ".git")
        .then(|| common.parent().map(Path::to_path_buf))
        .flatten()
}

#[cfg(test)]
mod tests;
