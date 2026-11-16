use super::{Effect, Finder, Found, Input, step};
use crate::{Action, Fuzzy, PickerRow};

pub(super) fn row(label: &str) -> PickerRow {
    PickerRow {
        label: label.to_string(),
        preview: None,
        enter: Action::Insert(label.to_string()),
        keys: Vec::new(),
        group: None,
    }
}

pub(super) fn finder_of(rows: Vec<PickerRow>) -> Finder<Fuzzy> {
    let (finder, _) = step(Finder::new(Fuzzy), Input::RowsArrived(rows));
    finder
}

pub(super) fn finder_with(labels: &[&str]) -> Finder<Fuzzy> {
    finder_of(labels.iter().map(|label| row(label)).collect())
}

pub(super) fn press(finder: Finder<Fuzzy>, keys: &[&str]) -> Finder<Fuzzy> {
    keys.iter().fold(finder, |finder, key| {
        step(finder, Input::Key(key.to_string())).0
    })
}

pub(super) fn typed(finder: Finder<Fuzzy>, text: &str) -> Finder<Fuzzy> {
    text.chars().fold(finder, |finder, character| {
        step(finder, Input::Key(character.to_string())).0
    })
}

pub(super) fn effects_of(finder: Finder<Fuzzy>, key: &str) -> Vec<Effect> {
    step(finder, Input::Key(key.to_string())).1
}

pub(super) fn found_by(finder: Finder<Fuzzy>, key: &str) -> Found {
    match effects_of(finder, key).as_slice() {
        [.., Effect::Close(found)] => found.clone(),
        effects => panic!("{key} did not close the finder: {effects:?}"),
    }
}
