use std::io::Write;

use herdr_picker_domain::PickerRow;

use super::page::notice;
use super::rows::RowWriter;
use super::{Asked, Places, written};
use crate::command::{CommandError, failed};

mod copy;
mod decode;
mod formats;
mod lines;
mod listing;
mod load;
mod place;
mod redact;
mod row;
mod sqlite;
mod tree;
mod views;

use formats::Format;
use lines::{Lines, lines};
use load::Origin;
use place::Place;
use row::Doc;

// The data picker and its steps, or None for a name that is not one of them.
pub(super) fn data_rows(
    name: &str,
    arguments: &[&str],
    asked: &Asked,
    places: &Places,
    rows: &mut RowWriter<impl Write>,
) -> Option<Result<(), CommandError>> {
    let place = asked.state.and_then(Place::parse).unwrap_or_default();
    let (header, listed) = match (name, arguments, asked.source) {
        ("data", [], Some("clipboard")) => document(&Origin::Clipboard, "paths", &place),
        ("data", [], Some("temporal")) => (None, vec![listing::temporal()]),
        ("data", [], _) => return Some(listing::data_files(places, rows)),
        ("data-file", [origin], source) => {
            document(&Origin::parse(origin), source.unwrap_or("paths"), &place)
        }
        ("data-copy", [origin], _) => match value(&Origin::parse(origin), &place) {
            Ok(value) => (
                Some(copy::copy_header(&place)),
                copy::copy_rows(&value, &place),
            ),
            Err(failure) => (None, vec![unreadable(&failure)]),
        },
        _ => return None,
    };
    let header = header.map_or(Ok(()), |header| rows.header(&header));
    Some(written(header.and_then(|()| {
        listed.iter().try_for_each(|row| rows.row(row))
    })))
}

fn document(origin: &Origin, source: &str, place: &Place) -> (Option<String>, Vec<PickerRow>) {
    if let (Origin::File(path), Some(Format::Sqlite)) = (origin, origin.format()) {
        return (None, sqlite::tables(path));
    }
    let value = match value(origin, place) {
        Ok(value) => value,
        Err(failure) => return (None, vec![unreadable(&failure)]),
    };
    let lines = match source {
        "shape" => Lines::Unknown,
        _ => lines(origin),
    };
    views::view(
        source,
        &Doc {
            origin,
            lines: &lines,
        },
        &value,
        place,
    )
}

// The value at a place in the document, or why there is none.
fn value(origin: &Origin, place: &Place) -> Result<serde_json::Value, String> {
    let document = origin.read()?;
    tree::at(&document, place)
        .ok_or_else(|| format!("Nothing is at {} now: the document changed.", place.label()))
}

fn unreadable(failure: &str) -> PickerRow {
    notice("Could not read this document", failure)
}

// herdr-picker data value ORIGIN PLACE: the value as a preview or an editor shows it.
pub(crate) fn data_command(arguments: &[&str]) -> Result<(), CommandError> {
    let ["value", origin, place] = arguments else {
        return Err(CommandError::Usage);
    };
    let place = Place::parse(place).ok_or_else(|| failed(format!("not a place: {place}")))?;
    let value = value(&Origin::parse(origin), &place).map_err(failed)?;
    print!("{}", tree::text(&value));
    Ok(())
}

#[cfg(test)]
mod tests;
