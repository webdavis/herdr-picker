use std::env;

use pwe_domain::{Action, Location, PickerCall, PickerRow, Preview};
use serde_json::Value;

use super::lines::Lines;
use super::load::Origin;
use super::place::Place;
use super::redact::redacted;
use super::tree::{container, summary, text};
use crate::builtin::page::{own_program, words};
use crate::builtin::paint::dim;
use crate::builtin::rows::key;

const LARGEST_VALUE_COPIED_FROM_THE_LIST: usize = 64 * 1024;

pub(super) struct Doc<'a> {
    pub(super) origin: &'a Origin,
    pub(super) lines: &'a Lines,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Enter {
    Types,
    Opens,
}

pub(super) fn path_label(place: &Place, value: &Value) -> String {
    format!("{}  {}{}", place.label(), dim(&summary(value)), tag(place))
}

pub(super) fn tag(place: &Place) -> String {
    match place.decoded() {
        true => format!("  {}", dim("decoded")),
        false => String::new(),
    }
}

pub(super) fn node_row(
    doc: &Doc,
    place: &Place,
    value: &Value,
    label: String,
    enter: Enter,
) -> PickerRow {
    let origin = doc.origin.argument();
    let state = place.state();
    let enter = match enter == Enter::Opens && container(value) {
        true => next("data-file", vec![origin.clone()], Some(state.clone())),
        false => Action::Insert(place.jq()),
    };
    let mut keys = vec![(key("ctrl-enter"), opened(doc, place, value))];
    let (shown, visible) = (text(&redacted(value, place.last_key())), text(value));
    if shown.len() <= LARGEST_VALUE_COPIED_FROM_THE_LIST {
        if shown == visible {
            keys.push((key("alt-y"), Action::CopyStay(shown.clone())));
        } else {
            let hidden = key("alt-y").labelled("copy and stay, secrets hidden");
            keys.push((hidden, Action::CopyStay(shown.clone())));
            let secrets = key("alt-u").labelled("copy and stay, with secrets");
            keys.push((secrets, Action::CopyStay(visible)));
        }
    }
    keys.push((
        key("alt-;").labelled("copy as JSON, YAML or another format"),
        next("data-copy", vec![origin.clone()], Some(state.clone())),
    ));
    let preview = match container(value) {
        true => Preview::Command(own_command(&["data", "value", &origin, &state])),
        false => Preview::Text(text(value)),
    };
    PickerRow {
        label,
        preview: Some(preview),
        enter,
        keys,
        group: None,
    }
}

fn opened(doc: &Doc, place: &Place, value: &Value) -> Action {
    let plain_text = value.as_str().is_some_and(|text| text.contains('\n'));
    match doc.origin {
        Origin::File(path) if !plain_text => Action::Edit(vec![Location {
            path: path.to_string_lossy().into_owned(),
            line: doc.lines.of(place),
        }]),
        origin => Action::Run(format!(
            "{} data value {} | nvim -",
            own_program(),
            words(&[&origin.argument(), &place.state()])
        )),
    }
}

pub(super) fn next(name: &str, args: Vec<String>, state: Option<String>) -> Action {
    Action::Next(PickerCall {
        name: name.to_string(),
        args,
        state,
        query: None,
    })
}

fn own_command(arguments: &[&str]) -> Vec<String> {
    let program = env::current_exe()
        .map_or_else(|_| "pwe".into(), |path| path.to_string_lossy().into_owned());
    std::iter::once(program)
        .chain(arguments.iter().map(|word| word.to_string()))
        .collect()
}
