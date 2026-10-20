use std::io::Write;

use herdr_picker_domain::{Action, PickerRow, Preview};

use super::paint::{dim, yellow};
use super::steps::next;
use super::{Repo, own_preview, strings};
use crate::builtin::rows::{RowWriter, key};
use crate::builtin::{Asked, Places, written};
use crate::command::CommandError;

mod state;

use state::Sort;

const COMMITS: &str = "--format=%H%x00%h%x00%s%x00%cr, %an%x00";
// A reflog entry says what moved HEAD and when, so it shows that instead of the subject.
const REFLOG: &str = "--format=%H%x00%h%x00%gs%x00%gd%x00";

pub(super) fn history(
    asked: &Asked,
    places: &Places,
    rows: &mut RowWriter<impl Write>,
) -> Result<(), CommandError> {
    let repo = Repo::at(&places.cwd)?;
    let sort = Sort::from_state(asked.state);
    let pickaxe = format!("-G{}", asked.query);
    let arguments: Vec<&str> = match asked.source.unwrap_or("commits") {
        "reflog" => vec!["reflog", "-500", "--date=relative", REFLOG],
        "changes" if asked.query.is_empty() => return Ok(()),
        "changes" => vec![
            "log",
            "-500",
            COMMITS,
            "--no-ext-diff",
            "--no-textconv",
            &pickaxe,
        ],
        _ => vec!["log", "-500", COMMITS],
    };
    let fields = strings(&repo.git(&arguments)?);
    let mut listed: Vec<PickerRow> = fields
        .as_chunks::<4>()
        .0
        .iter()
        .map(|commit| commit_row(&repo, commit, sort))
        .collect();
    if sort == Sort::OldestFirst {
        listed.reverse();
    }
    written(listed.iter().try_for_each(|row| rows.row(row)))
}

fn commit_row(repo: &Repo, fields: &[String; 4], sort: Sort) -> PickerRow {
    let [commit, short, subject, when] = [0, 1, 2, 3].map(|index| fields[index].trim().to_string());
    let mut preview = repo.command(&["show", "--color=always", "--no-ext-diff", "--no-textconv"]);
    preview.args(["--stat", "--patch", &commit]);
    PickerRow {
        label: format!("{} {subject}  {}", yellow(&short), dim(&when)),
        preview: Some(Preview::Command(super::status::words(&preview))),
        enter: Action::Insert(commit.clone()),
        keys: vec![
            (
                key("ctrl-enter"),
                next("checkout", &["commit", &commit], None, None),
            ),
            (key("ctrl-t"), next("commit-files", &[&commit], None, None)),
            (key("alt-s"), Action::State(sort.flipped().state())),
            (key("alt-i"), Action::Run(repo.shell(&["show", &commit]))),
        ],
        group: None,
    }
}

// The files of one commit, each read as it was then.
pub(super) fn commit_files(
    commit: &str,
    places: &Places,
    rows: &mut RowWriter<impl Write>,
) -> Result<(), CommandError> {
    let repo = Repo::at(&places.cwd)?;
    let about =
        String::from_utf8_lossy(&repo.git(&["log", "-1", "--format=%h%x09%s%x09%cr", commit])?)
            .trim_end()
            .to_string();
    let mut parts = about.splitn(3, '\t');
    let (short, subject, age) = (parts.next(), parts.next(), parts.next());
    written(rows.header(&format!(
        "{} {} ({})",
        short.unwrap_or_default(),
        subject.unwrap_or_default(),
        age.unwrap_or_default()
    )))?;
    let root = repo.root.to_string_lossy().into_owned();
    let tree = repo.git(&["ls-tree", "-rz", commit])?;
    let listed: Vec<PickerRow> = strings(&tree)
        .iter()
        .filter_map(|item| {
            let (metadata, name) = item.split_once('\t')?;
            let mut fields = metadata.split(' ');
            let (kind, blob) = (fields.nth(1)?, fields.next()?);
            let read_it = format!("{} | nvim -R -", repo.shell(&["cat-file", "blob", blob]));
            (kind == "blob").then(|| PickerRow {
                label: name.to_string(),
                preview: Some(Preview::Command(own_preview(&["blob", &root, blob, name]))),
                enter: Action::Insert(read_it.clone()),
                keys: vec![(key("ctrl-enter"), Action::Run(read_it))],
                group: None,
            })
        })
        .collect();
    written(listed.iter().try_for_each(|row| rows.row(row)))
}

#[cfg(test)]
mod tests;
