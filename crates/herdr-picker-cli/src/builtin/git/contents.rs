use std::io::Write;
use std::path::{Path, PathBuf};

use herdr_picker_domain::{Action, PickerRow, Preview};

use super::paint::{blue, dim, yellow};
use super::{Repo, strings};
use crate::builtin::rows::{RowWriter, file_row, key};
use crate::builtin::{Asked, Places, written};
use crate::command::CommandError;

mod area;
mod search;
mod state;

use area::area;

use search::{Hit, names, search};
use state::Switches;

// Lines matching what is typed, in the repository's tracked or untracked files, or in every
// file here outside one.
pub(super) fn contents(
    asked: &Asked,
    places: &Places,
    rows: &mut RowWriter<impl Write>,
) -> Result<(), CommandError> {
    let switches = Switches::read(asked.state);
    let start = switches
        .worktree
        .as_ref()
        .map_or_else(|| places.cwd.clone(), PathBuf::from);
    let repo = Repo::at(&start).ok();
    let root = repo.as_ref().map_or(start, |repo| repo.root.clone());
    let files = match (&repo, switches.untracked) {
        (None, _) => "the files here",
        (Some(_), false) => "the tracked files",
        (Some(_), true) => "the untracked files",
    };
    let mut say = |text: String| written(rows.row(&notice(&text, &switches, repo.is_some())));
    if asked.query.is_empty() {
        return say(format!("Type to search {files}"));
    }
    let listed = repo.as_ref().map(|_| names(&root, &switches)).transpose()?;
    let mut hits = search(&root, listed.as_deref(), asked.query, &switches)?;
    if switches.sort {
        let modified = |hit: &Hit| {
            root.join(&hit.name)
                .metadata()
                .and_then(|found| found.modified())
                .ok()
        };
        hits.sort_by_key(|hit| std::cmp::Reverse((modified(hit), hit.line)));
    }
    let changed = repo.as_ref().map(|repo| {
        strings(
            &repo
                .git(&["diff", "--name-only", "-z", "HEAD"])
                .unwrap_or_default(),
        )
    });
    if hits.is_empty() {
        return say(format!("No line in {files} matches {}", asked.query));
    }
    let listed: Vec<PickerRow> = hits
        .iter()
        .map(|hit| {
            hit_row(
                hit,
                &root,
                repo.as_ref(),
                changed.as_deref(),
                &switches,
                places,
            )
        })
        .collect();
    written(listed.iter().try_for_each(|row| rows.row(row)))
}

fn hit_row(
    hit: &Hit,
    root: &Path,
    repo: Option<&Repo>,
    changed: Option<&[String]>,
    switches: &Switches,
    places: &Places,
) -> PickerRow {
    let path = root.join(&hit.name);
    let label = format!(
        "{}:{}: {}",
        blue(&hit.name),
        yellow(&hit.line.to_string()),
        hit.text
    );
    let mut row = file_row(&path, label, places, None);
    row.enter = super::status::edit_at(&path, Some(hit.line));
    if let Some(bat) = &places.bat {
        let from = hit.line.saturating_sub(10).max(1);
        row.preview = Some(Preview::Command(vec![
            bat.to_string_lossy().into_owned(),
            "--color=always".into(),
            "--style=numbers".into(),
            format!("--highlight-line={}", hit.line),
            format!("--line-range={from}:"),
            path.to_string_lossy().into_owned(),
        ]));
    }
    row.group = Some(area(&hit.name).to_string());
    if let Some(repo) = repo {
        let committed =
            !switches.untracked && changed.is_some_and(|changed| !changed.contains(&hit.name));
        if committed {
            let range = format!("{0},{0}:{1}", hit.line, hit.name);
            row.keys.push((
                key("alt-g"),
                Action::Run(repo.shell(&["log", "-L", &range])),
            ));
        }
    }
    row.keys.extend(switches.keys(repo.is_some()));
    row
}

// With nothing to list, one row says why and keeps the switches in reach.
fn notice(text: &str, switches: &Switches, in_git: bool) -> PickerRow {
    PickerRow {
        label: dim(text),
        preview: None,
        enter: Action::Nothing,
        keys: switches.keys(in_git),
        group: None,
    }
}

#[cfg(test)]
mod tests;
