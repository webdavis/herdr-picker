use std::io::Write;
use std::path::{Path, PathBuf};

use super::git::Repo;
use super::page::{command_row, notice, words};
use super::rows::RowWriter;
use super::{Places, finder_closing_is_no_failure};
use crate::command::CommandError;

mod capture;
mod diagnostics;
mod discovered;
mod errors;
mod failures;
mod frameworks;
mod packages;
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
    finder_closing_is_no_failure(listed.iter().try_for_each(|row| rows.row(row)))
}

fn root(cwd: &Path) -> PathBuf {
    Repo::at(cwd).map_or_else(|_| cwd.to_path_buf(), |repo| repo.root)
}

fn nearest(cwd: &Path, names: &[&str]) -> Option<PathBuf> {
    cwd.ancestors()
        .flat_map(|folder| names.iter().map(move |name| folder.join(name)))
        .find(|path| path.is_file())
}
