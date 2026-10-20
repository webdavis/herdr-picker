use herdr_picker_domain::{Action, PickerRow, Preview};

use super::{Person, chosen, named, readable};
use crate::builtin::paint::{blue, dim, green, mauve, peach};

// Each of the contacts' fields, Enter copying its value.
pub(in crate::builtin::apple) fn field_rows(
    people: &[Person],
    ids: &[&str],
    offset: i64,
) -> Vec<PickerRow> {
    chosen(people, ids)
        .flat_map(|person| {
            readable(person, offset)
                .into_iter()
                .map(|(field, value)| PickerRow {
                    label: format!(
                        "{} {}",
                        dim(&format!("{}:", field.label)),
                        painted(&field.group, &value)
                    ),
                    preview: Some(Preview::Text(format!(
                        "{} of {}",
                        field.label,
                        named(person)
                    ))),
                    enter: Action::Copy(value),
                    keys: Vec::new(),
                    group: Some(field.group.clone()),
                })
        })
        .collect()
}

// Each of the contacts' phone numbers, Enter calling it.
pub(in crate::builtin::apple) fn call_rows(
    people: &[Person],
    ids: &[&str],
    offset: i64,
) -> Vec<PickerRow> {
    let rows = field_rows(people, ids, offset);
    rows.into_iter()
        .filter(|row| row.group.as_deref() == Some("phone"))
        .map(|row| {
            let Action::Copy(number) = &row.enter else {
                return row;
            };
            match tel_url(number) {
                Some(url) => PickerRow {
                    enter: Action::Open(url),
                    ..row
                },
                None => PickerRow {
                    enter: Action::Nothing,
                    preview: Some(Preview::Text(
                        "The selected phone number cannot be used as a telephone URL.".to_string(),
                    )),
                    ..row
                },
            }
        })
        .collect()
}

pub(super) fn tel_url(number: &str) -> Option<String> {
    let number: String = number
        .chars()
        .filter(|c| !c.is_whitespace() && !"().-".contains(*c))
        .collect();
    let digits = number.strip_prefix('+').unwrap_or(&number);
    let valid = digits
        .split([';', ','])
        .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()));
    valid.then(|| format!("tel:{number}"))
}

fn painted(group: &str, value: &str) -> String {
    match group {
        "phone" => green(value),
        "email" | "link" => blue(value),
        "address" => peach(value),
        "work" => mauve(value),
        _ => value.to_string(),
    }
}
