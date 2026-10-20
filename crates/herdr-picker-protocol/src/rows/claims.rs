use std::collections::BTreeMap;

use herdr_picker_domain::{Action, Key};

use super::action::into_action;
use super::wire::{WireAction, Words};

pub(super) fn claimed_keys(
    keys: BTreeMap<String, WireAction>,
    labels: BTreeMap<String, String>,
    value: Option<&Words>,
) -> Result<Vec<(Key, Action)>, String> {
    let mut claimed: Vec<(Key, Action)> = Vec::new();
    for (name, action) in keys {
        let key = claimable(&name)?;
        if claimed.iter().any(|(other, _)| *other == key) {
            return Err(format!("{name} is a key the row already claims"));
        }
        claimed.push((key, into_action(action, value)?));
    }
    for (name, label) in labels {
        let labelled =
            Key::parse(&name).and_then(|named| claimed.iter_mut().find(|(key, _)| *key == named));
        let Some((key, _)) = labelled else {
            return Err(format!(
                "{name} has a label, but the row claims no such key"
            ));
        };
        *key = key.clone().labelled(&label);
    }
    Ok(claimed)
}

fn claimable(name: &str) -> Result<Key, String> {
    match Key::parse(name) {
        Some(key) if key != Key::enter() => Ok(key),
        Some(_) => Err("enter belongs in the row's enter field".to_string()),
        None => Err(format!("{name} is not a key a row can claim")),
    }
}
