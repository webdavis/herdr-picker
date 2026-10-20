use std::fs;
use std::io::Write;

use herdr_picker_adapters::read_picker_config;
use herdr_picker_domain::{Action, Picker, PickerCall, PickerCommand, PickerRow, Preview};

use super::paint::dim;
use super::rows::RowWriter;
use super::{Places, written};
use crate::command::{CommandError, failed, picker_config_path};
use table::{Binding, table_bindings};

const CHORD_TABLE: &str = ".config/chord/bindings-menu.tsv";
const OWN_GROUP: &str = "herdr-picker";

// The pickers on their keys, or every key binding: herdr-picker's, then the chord table's.
pub(super) fn catalog(
    source: &str,
    places: &Places,
    rows: &mut RowWriter<impl Write>,
) -> Result<(), CommandError> {
    let pickers = read_picker_config(&picker_config_path())
        .map_err(failed)?
        .pickers;
    let table = places
        .home
        .as_ref()
        .and_then(|home| fs::read_to_string(home.join(CHORD_TABLE)).ok())
        .unwrap_or_default();
    written(
        catalog_rows(source, &pickers, &table)
            .iter()
            .try_for_each(|row| rows.row(row)),
    )
}

pub(super) fn catalog_rows(source: &str, pickers: &[Picker], table: &str) -> Vec<PickerRow> {
    if source == "pickers" {
        return picker_rows(pickers);
    }
    let own = pickers.iter().map(|picker| Binding {
        key: picker.chord.to_string(),
        group: OWN_GROUP,
        kind: "picker",
        action: &picker.name,
        description: &picker.description,
        enter: start(picker),
    });
    let bindings: Vec<Binding> = own.chain(table_bindings(table)).collect();
    let width = widest(bindings.iter().map(|binding| binding.key.as_str()));
    bindings
        .into_iter()
        .map(|binding| {
            let shown = Some(binding.description).filter(|text| !text.is_empty());
            let details = format!(
                "Key     {}\nKind    {}\nAction  {}",
                binding.key, binding.kind, binding.action
            );
            PickerRow {
                label: format!(
                    "{:<width$}  {}",
                    binding.key,
                    shown.unwrap_or(binding.action)
                ),
                preview: Some(Preview::Text(match shown {
                    Some(description) => format!("{description}\n\n{details}"),
                    None => details,
                })),
                enter: binding.enter,
                keys: Vec::new(),
                group: Some(binding.group.to_string()),
            }
        })
        .collect()
}

fn picker_rows(pickers: &[Picker]) -> Vec<PickerRow> {
    let keys: Vec<String> = pickers
        .iter()
        .map(|picker| picker.chord.to_string())
        .collect();
    let key_width = widest(keys.iter().map(String::as_str));
    let name_width = widest(pickers.iter().map(|picker| picker.name.as_str()));
    pickers
        .iter()
        .zip(keys)
        .map(|(picker, key)| PickerRow {
            label: format!(
                "{key:<key_width$}  {:<name_width$}  {}",
                picker.name,
                dim(&picker.description)
            ),
            preview: Some(Preview::Text(format!(
                "{}\n\nKey     {key}\nPicker  {}",
                picker.description, picker.name
            ))),
            enter: start(picker),
            keys: Vec::new(),
            group: None,
        })
        .collect()
}

// The bookmark runs no finder, and this picker is already open.
fn start(picker: &Picker) -> Action {
    match &picker.command {
        PickerCommand::Builtin(name) if name == "bookmark" || name == "pickers" => Action::Nothing,
        _ => Action::Next(PickerCall {
            name: picker.name.clone(),
            args: Vec::new(),
            state: None,
            query: None,
        }),
    }
}

fn widest<'a>(texts: impl Iterator<Item = &'a str>) -> usize {
    texts
        .map(|text| text.chars().count())
        .max()
        .unwrap_or_default()
}

mod table;

#[cfg(test)]
mod tests;
