use std::io::Write;
use std::path::{Path, PathBuf};

use herdr_picker_domain::{Action, PickerRow, Preview};

use super::git::Repo;
use super::paint::dim;
use super::rows::{RowWriter, key};
use super::{Places, written};
use crate::command::CommandError;

mod capture;
mod diagnostics;
mod errors;
mod saved;
mod suites;

pub(crate) use capture::save_check;

const JUSTFILES: [&str; 3] = ["justfile", "Justfile", ".justfile"];

pub(super) fn tests(
    source: &str,
    places: &Places,
    rows: &mut RowWriter<impl Write>,
) -> Result<(), CommandError> {
    let listed = match source {
        "errors" => errors::errors(places)?,
        "quickfix" => saved::saved(places),
        _ => suites::suites(places)?,
    };
    written(listed.iter().try_for_each(|row| rows.row(row)))
}

// The folder the project's files hang from: its repository's, or the one the picker runs in.
fn root(cwd: &Path) -> PathBuf {
    Repo::at(cwd).map_or_else(|_| cwd.to_path_buf(), |repo| repo.root)
}

// The first of the files in this folder or a folder above it.
fn nearest(cwd: &Path, names: &[&str]) -> Option<PathBuf> {
    cwd.ancestors()
        .flat_map(|folder| names.iter().map(move |name| folder.join(name)))
        .find(|path| path.is_file())
}

fn notice(label: &str, detail: &str) -> PickerRow {
    PickerRow {
        label: dim(label),
        preview: Some(Preview::Text(detail.to_string())),
        enter: Action::Nothing,
        keys: Vec::new(),
        group: None,
    }
}

// Enter prepares the command at the prompt; Ctrl-Enter runs it now.
fn command_row(label: String, command: String, detail: &str) -> PickerRow {
    PickerRow {
        label,
        preview: Some(Preview::Text(format!(
            "{command}\n\n{detail}\nEnter prepares this command."
        ))),
        enter: Action::Insert(command.clone()),
        keys: vec![(key("ctrl-enter"), Action::Run(command))],
        group: None,
    }
}

fn words(words: &[&str]) -> String {
    words
        .iter()
        .map(|word| herdr_picker_protocol::shell_word(word))
        .collect::<Vec<_>>()
        .join(" ")
}
