use herdr_picker_domain::{Action, PickerRow, Preview};
use serde::Deserialize;
use serde_json::json;

use super::step;
use crate::builtin::clock::{parse_iso, stamp};
use crate::builtin::rows::key;

mod fields;
pub(super) mod people;

use fields::tel_url;
pub(super) use fields::{call_rows, field_rows};

#[derive(Debug, Clone, Deserialize)]
pub(super) struct Person {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) fields: Vec<Field>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct Field {
    pub(super) group: String,
    pub(super) label: String,
    pub(super) value: String,
}

// Alt-0 to Alt-6 choose which of a contact's fields Enter copies.
const MODES: [&str; 7] = ["all", "name", "phone", "email", "address", "work", "link"];

pub(super) fn contact_rows(people: &[Person], mode: &str, offset: i64) -> Vec<PickerRow> {
    people
        .iter()
        .map(|person| contact_row(person, mode, offset))
        .collect()
}

fn contact_row(person: &Person, mode: &str, offset: i64) -> PickerRow {
    let fields = readable(person, offset);
    let card = lines(
        fields
            .iter()
            .map(|(field, value)| format!("{}: {value}", field.label)),
    );
    let copied = match mode {
        "all" => card.clone(),
        _ => lines(
            fields
                .iter()
                .filter(|(field, _)| field.group == mode)
                .map(|(_, value)| value.clone()),
        ),
    };
    let mut keys = vec![(key("alt-y"), Action::CopyStay(copied.clone()))];
    keys.extend(MODES.iter().enumerate().map(|(digit, mode)| {
        (
            key(&format!("alt-{digit}")),
            Action::State(json!(mode).to_string()),
        )
    }));
    keys.push((
        key("alt-7"),
        step("contact-fields", vec![person.id.clone()]),
    ));
    let phones: Vec<&str> = in_group(person, "phone").collect();
    match phones.as_slice() {
        [] => {}
        [phone] => keys.push((
            key("ctrl-v"),
            tel_url(phone).map_or(Action::Nothing, Action::Open),
        )),
        _ => keys.push((key("ctrl-v"), step("contact-call", vec![person.id.clone()]))),
    }
    let addresses: Vec<String> = in_group(person, "phone")
        .chain(in_group(person, "email"))
        .map(str::to_string)
        .collect();
    if !addresses.is_empty() {
        keys.push((key("alt-m"), step("chats", addresses)));
    }
    PickerRow {
        label: named(person).to_string(),
        preview: Some(Preview::Text(format!(
            "{card}\n\nCopy: {mode}. Enter copies; Alt-Y copies and stays."
        ))),
        enter: Action::Copy(copied),
        keys,
        group: None,
    }
}

pub(super) fn chosen<'a>(
    people: &'a [Person],
    ids: &'a [&str],
) -> impl Iterator<Item = &'a Person> {
    people
        .iter()
        .filter(|person| ids.contains(&person.id.as_str()))
}

pub(super) fn named(person: &Person) -> &str {
    if person.name.is_empty() {
        "(unnamed contact)"
    } else {
        &person.name
    }
}

fn in_group<'a>(person: &'a Person, group: &'a str) -> impl Iterator<Item = &'a str> {
    person
        .fields
        .iter()
        .filter(move |field| field.group == group)
        .map(|field| field.value.as_str())
}

// The non-empty fields, Contacts' ISO creation and change times shown as local times.
pub(super) fn readable(person: &Person, offset: i64) -> Vec<(&Field, String)> {
    person
        .fields
        .iter()
        .filter(|field| !field.value.trim().is_empty())
        .map(|field| {
            let when = matches!(field.label.as_str(), "Created" | "Modified")
                .then(|| parse_iso(&field.value))
                .flatten();
            (
                field,
                when.map_or_else(|| field.value.clone(), |epoch| stamp(epoch, offset)),
            )
        })
        .collect()
}

fn lines(lines: impl Iterator<Item = String>) -> String {
    lines.collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests;
