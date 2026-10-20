use std::borrow::Cow;

use serde_json::Value;

use super::decode::{base64, decoded};
use super::place::{Decoding, Place, Segment};
use crate::builtin::git::plural;

const SUMMARY_WIDTH: usize = 60;

// A value where it sits, a string holding JSON or base64 text read in place as what it holds.
pub(super) fn in_place(value: &Value, place: Place) -> (Place, Cow<'_, Value>) {
    match value.as_str().and_then(decoded) {
        Some((decodings, held)) => {
            let place = decodings.into_iter().fold(place, |place, decoding| {
                place.child(Segment::Decoded(decoding))
            });
            (place, Cow::Owned(held))
        }
        None => (place, Cow::Borrowed(value)),
    }
}

// Every value from this one down, each before the values inside it.
pub(super) fn walk(value: &Value, place: Place, visit: &mut impl FnMut(&Place, &Value)) {
    let (place, value) = in_place(value, place);
    visit(&place, &value);
    for (segment, child) in inside(&value) {
        walk(child, place.child(segment), visit);
    }
}

// The values one level inside this one, each read in place.
pub(super) fn children(value: &Value, place: &Place) -> Vec<(Place, Value)> {
    inside(value)
        .map(|(segment, child)| {
            let (place, child) = in_place(child, place.child(segment));
            (place, child.into_owned())
        })
        .collect()
}

pub(super) fn at(root: &Value, place: &Place) -> Option<Value> {
    let mut value = Cow::Borrowed(root);
    for segment in &place.0 {
        value = match segment {
            Segment::Key(key) => step(value, |value| value.get(key.as_str()))?,
            Segment::Index(index) => step(value, |value| value.get(*index))?,
            Segment::Decoded(Decoding::Json) => {
                Cow::Owned(serde_json::from_str(value.as_str()?.trim()).ok()?)
            }
            Segment::Decoded(Decoding::Base64) => Cow::Owned(Value::String(
                String::from_utf8(base64(value.as_str()?.trim())?).ok()?,
            )),
        };
    }
    Some(value.into_owned())
}

fn step<'a>(
    value: Cow<'a, Value>,
    inner: impl Fn(&Value) -> Option<&Value>,
) -> Option<Cow<'a, Value>> {
    match value {
        Cow::Borrowed(value) => inner(value).map(Cow::Borrowed),
        Cow::Owned(value) => inner(&value).cloned().map(Cow::Owned),
    }
}

fn inside(value: &Value) -> Box<dyn Iterator<Item = (Segment, &Value)> + '_> {
    match value {
        Value::Object(fields) => Box::new(
            fields
                .iter()
                .map(|(key, field)| (Segment::Key(key.clone()), field)),
        ),
        Value::Array(items) => Box::new(
            items
                .iter()
                .enumerate()
                .map(|(index, item)| (Segment::Index(index), item)),
        ),
        _ => Box::new(std::iter::empty()),
    }
}

pub(super) fn container(value: &Value) -> bool {
    matches!(value, Value::Object(_) | Value::Array(_))
}

// A string as its text, line breaks and all; anything else as indented JSON.
pub(super) fn text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => serde_json::to_string_pretty(other).unwrap_or_default(),
    }
}

// The value in a few words on one line.
pub(super) fn summary(value: &Value) -> String {
    let shown = match value {
        Value::Object(fields) => format!("{{{}}}", plural(fields.len(), "key")),
        Value::Array(items) => format!("[{}]", plural(items.len(), "item")),
        other => other.to_string(),
    };
    match shown.chars().count() > SUMMARY_WIDTH {
        true => format!("{}…", shown.chars().take(SUMMARY_WIDTH).collect::<String>()),
        false => shown,
    }
}
