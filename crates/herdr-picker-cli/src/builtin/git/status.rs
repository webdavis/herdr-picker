use std::io::Write;
use std::path::Path;

use herdr_picker_domain::{Action, Location, PickerRow, Preview};

use super::paint::{dim, yellow};
use super::steps::{confirm, next};
use super::{Repo, own_preview, strings};
use crate::builtin::rows::{RowWriter, file_row, key, painted, shown};
use crate::builtin::{Places, written};
use crate::command::CommandError;

mod ignored;
mod porcelain;
mod summary;

use porcelain::{File, Porcelain};
use summary::summary;

pub(super) fn status(
    source: &str,
    places: &Places,
    rows: &mut RowWriter<impl Write>,
) -> Result<(), CommandError> {
    let repo = Repo::at(&places.cwd)?;
    if source == "ignored" {
        return ignored::ignored(&repo, places, rows);
    }
    let porcelain = Porcelain::read(&repo)?;
    written(rows.header(&summary(&porcelain)))?;
    let stashes = match source {
        "all" | "stashes" => stashes(&repo)?,
        _ => Vec::new(),
    };
    let shown_files = |quiet: bool| {
        porcelain.files.iter().filter(move |file| match source {
            "all" => (file.states[0] == "tracked") == quiet,
            "stashes" => false,
            state => !quiet && file.states.contains(&state),
        })
    };
    let all = source == "all";
    let file_rows = |quiet| shown_files(quiet).map(|file| file_row_of(&repo, file, all, places));
    let listed: Vec<PickerRow> = file_rows(false)
        .chain(stashes)
        .chain(file_rows(true))
        .collect();
    written(listed.iter().try_for_each(|row| rows.row(row)))
}

fn file_row_of(repo: &Repo, file: &File, all: bool, places: &Places) -> PickerRow {
    let path = repo.root.join(&file.name);
    let shown_path = painted(&shown(&path, &places.cwd));
    let label = match all {
        true => format!("{}  {shown_path}", dim(&format!("{:<9}", file.states[0]))),
        false => shown_path,
    };
    let mut row = file_row(&path, label, places, None);
    let root = repo.root.to_string_lossy();
    row.preview = Some(match &file.stages {
        Some(stages) => Preview::Command(own_preview(&[
            "stages", &root, &file.name, &stages[0], &stages[1], &stages[2],
        ])),
        None if file.changed() => Preview::Command(own_preview(&["diff", &root, &file.name])),
        None => Preview::Path(path.to_string_lossy().into_owned()),
    });
    if file.stashable() {
        let stash = next("stash-files", &[&file.name], None, None);
        row.keys.push((key("alt-z"), stash));
    }
    row
}

fn stashes(repo: &Repo) -> Result<Vec<PickerRow>, CommandError> {
    let fields = strings(&repo.git(&["stash", "list", "--format=%gd%x00%gs%x00"])?);
    Ok(fields
        .as_chunks::<2>()
        .0
        .iter()
        .map(|[name, subject]| {
            let name = name.trim();
            let apply = repo.shell(&["stash", "apply", name]);
            let mut preview = repo.command(&["stash", "show", "--color=always", "--patch"]);
            preview.args(["--include-untracked", name]);
            PickerRow {
                label: format!("{} {subject}", yellow(name)),
                preview: Some(Preview::Command(words(&preview))),
                enter: Action::Insert(name.to_string()),
                keys: vec![(
                    key("ctrl-enter"),
                    confirm(&format!("Apply {name} ({subject})?"), apply),
                )],
                group: None,
            }
        })
        .collect())
}

pub(super) fn words(command: &std::process::Command) -> Vec<String> {
    std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|word| word.to_string_lossy().into_owned())
        .collect()
}

pub(super) fn edit_at(path: &Path, line: Option<u32>) -> Action {
    Action::Edit(vec![Location {
        path: path.to_string_lossy().into_owned(),
        line,
    }])
}

#[cfg(test)]
mod tests;
