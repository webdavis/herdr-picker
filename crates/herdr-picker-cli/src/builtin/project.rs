use std::io::Write;
use std::path::{Path, PathBuf};

use super::git::Repo;
use super::page::{command_row, notice, words};
use super::rows::RowWriter;
use super::{Places, written};
use crate::command::CommandError;

mod capture;
mod diagnostics;
mod errors;
mod failures;
mod saved;
mod suites;

pub(crate) use capture::save_check;
pub(super) use failures::failures;

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
