use std::time::Duration;

use herdr_picker_domain::{Action, PickerRow, Preview};
use serde_json::Value;

use super::formats::Output;
use super::load::{NO_TOON, toon_installed};
use super::place::Place;
use super::redact::redacted;
use crate::builtin::page::notice;
use crate::builtin::program::ran;
use crate::builtin::rows::key;

const DEADLINE: Duration = Duration::from_secs(10);

// Alt-;'s menu: the value in each format, its secrets hidden unless copied with Alt-U.
pub(super) fn copy_rows(value: &Value, place: &Place) -> Vec<PickerRow> {
    let shown = redacted(value, place.last_key());
    Output::ALL
        .iter()
        .map(|output| {
            if *output == Output::Toon && !toon_installed() {
                return notice(output.name(), NO_TOON);
            }
            match (converted(*output, &shown), converted(*output, value)) {
                (Ok(hidden), Ok(visible)) => {
                    let mut keys = Vec::new();
                    if hidden != visible {
                        keys.push((
                            key("alt-u").labelled("copy with secrets"),
                            Action::Copy(visible),
                        ));
                    }
                    PickerRow {
                        label: output.name().to_string(),
                        preview: Some(Preview::Text(hidden.clone())),
                        enter: Action::Copy(hidden),
                        keys,
                        group: None,
                    }
                }
                (Err(failure), _) | (_, Err(failure)) => notice(output.name(), &failure),
            }
        })
        .collect()
}

pub(super) fn copy_header(place: &Place) -> String {
    format!(
        "Copy {} as: secrets hidden, Alt-U copies them visible.",
        place.label()
    )
}

pub(super) fn converted(output: Output, value: &Value) -> Result<String, String> {
    let Some(argv) = output.converter() else {
        return serde_json::to_string_pretty(value).map_err(|failure| failure.to_string());
    };
    let done = ran(argv, Some(&value.to_string()), DEADLINE)?;
    match done.succeeded {
        true => Ok(done.stdout.trim_end().to_string()),
        false => Err(format!("{} failed: {}", argv[0], done.stderr.trim())),
    }
}

#[cfg(test)]
mod tests;
