use std::io::Write;

use pwe_adapters::{Entry, Walk, walk};
use pwe_domain::PickerRow;

use super::formats::Format;
use super::row::next;
use crate::builtin::page::notice;
use crate::builtin::paint::dim;
use crate::builtin::rows::{RowWriter, file_row, painted, shown};
use crate::builtin::{Places, finder_closing_is_no_failure};
use crate::command::CommandError;

pub(super) const NO_TEMPORAL: &str = "Temporal picker not built yet";

pub(super) fn data_files(
    places: &Places,
    rows: &mut RowWriter<impl Write>,
) -> Result<(), CommandError> {
    let search = Walk {
        root: places.cwd.clone(),
        entry: Entry::Files,
        hidden: false,
        depth: None,
    };
    let mut found = 0;
    finder_closing_is_no_failure(walk(&search, places.fd.as_deref(), |path| {
        let Some(format) = Format::of(path) else {
            return Ok(());
        };
        found += 1;
        let label = format!(
            "{}  {}",
            painted(&shown(path, &places.cwd)),
            dim(format.name())
        );
        let full = path.to_string_lossy().into_owned();
        rows.row(&PickerRow {
            enter: next("data-file", vec![full], None),
            ..file_row(path, label, places, None)
        })
    }))?;
    if found == 0 {
        finder_closing_is_no_failure(rows.row(&notice(
            "No data files here",
            "JSON, JSON Lines, YAML, TOON and SQLite files under this folder show here.",
        )))?;
    }
    Ok(())
}

pub(super) fn temporal() -> PickerRow {
    notice(
        NO_TEMPORAL,
        "The Temporal picker is a later picker of its own.",
    )
}
