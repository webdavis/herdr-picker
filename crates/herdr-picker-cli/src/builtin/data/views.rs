use std::collections::HashSet;

use herdr_picker_domain::{Action, PickerRow};
use serde_json::Value;

use super::place::{Place, Segment};
use super::row::{Doc, Enter, node_row, path_label, tag};
use super::tree::{children, container, summary, walk};
use crate::builtin::page::notice;
use crate::builtin::paint::{blue, dim};

pub(super) const SHAPE_NOTE: &str =
    "Each field once, with an example value. A first version: its display is still to be designed.";

// A document's rows as a source shows them, from the place the step opened at; a header if any.
pub(super) fn view(
    source: &str,
    doc: &Doc,
    value: &Value,
    place: &Place,
) -> (Option<String>, Vec<PickerRow>) {
    let mut rows = Vec::new();
    let header = match source {
        "zoom" => {
            walk(value, place.clone(), &mut |place, value| {
                rows.push(node_row(
                    doc,
                    place,
                    value,
                    path_label(place, value),
                    Enter::Opens,
                ));
            });
            Some(format!(
                "{}  Enter on an object or array zooms in; Esc comes back.",
                place.label()
            ))
        }
        "levels" => {
            rows = levels(doc, value, place);
            Some(place.label())
        }
        "keys" => {
            rows = keys(doc, value, place);
            None
        }
        "shape" => {
            rows = shape(doc, value, place);
            Some(SHAPE_NOTE.to_string())
        }
        "tables" => {
            rows.push(notice(
                "No tables: this is not a SQLite database",
                "The tables source lists a SQLite database's tables, views and indexes.",
            ));
            None
        }
        _ => {
            walk(value, place.clone(), &mut |place, value| {
                rows.push(node_row(
                    doc,
                    place,
                    value,
                    path_label(place, value),
                    Enter::Types,
                ));
            });
            None
        }
    };
    (header, rows)
}

// Like folders: the keys one level down, an object or array opening as the next level.
fn levels(doc: &Doc, value: &Value, place: &Place) -> Vec<PickerRow> {
    children(value, place)
        .iter()
        .map(|(child, value)| {
            let name = match child
                .0
                .iter()
                .rev()
                .find(|segment| !matches!(segment, Segment::Decoded(_)))
            {
                Some(Segment::Key(key)) => key.clone(),
                Some(Segment::Index(index)) => format!("[{index}]"),
                _ => ".".to_string(),
            };
            let name = match container(value) {
                true => blue(&format!("{name}/")),
                false => name,
            };
            let label = format!("{name}  {}{}", dim(&summary(value)), tag(child));
            node_row(doc, child, value, label, Enter::Opens)
        })
        .collect()
}

// Every place a key appears, the places of one key together.
fn keys(doc: &Doc, value: &Value, place: &Place) -> Vec<PickerRow> {
    let mut found: Vec<(String, PickerRow)> = Vec::new();
    walk(value, place.clone(), &mut |place, value| {
        if let Some(Segment::Key(key)) = place.0.last() {
            let label = format!(
                "{key}  {}  {}{}",
                place.label(),
                dim(&summary(value)),
                tag(place)
            );
            found.push((
                key.clone(),
                node_row(doc, place, value, label, Enter::Types),
            ));
        }
    });
    found.sort_by(|(one, _), (other, _)| one.cmp(other));
    found.into_iter().map(|(_, row)| row).collect()
}

// Each field once, every item of an array counted as one, with the first value met as its example.
fn shape(doc: &Doc, value: &Value, place: &Place) -> Vec<PickerRow> {
    let mut seen = HashSet::new();
    let mut rows = Vec::new();
    walk(value, place.clone(), &mut |place, value| {
        let shape = place.shape();
        if seen.insert(shape.clone()) {
            let label = format!("{shape}  {}{}", dim(&summary(value)), tag(place));
            rows.push(PickerRow {
                enter: Action::Insert(place.shape_jq()),
                ..node_row(doc, place, value, label, Enter::Types)
            });
        }
    });
    rows
}

#[cfg(test)]
mod tests;
