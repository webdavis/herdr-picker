use crate::chord::{Chord, Stroke};
use crate::keymap::{Keymap, Legend, PickKey, ReadlineKeymap};
use crate::pick::{Command, Mode};
use crate::picker::Picker;
use crate::target::JumpTarget;

pub fn launcher_targets(pickers: &[Picker], base: Stroke) -> Vec<JumpTarget> {
    let mut targets: Vec<JumpTarget> = pickers
        .iter()
        .filter_map(|picker| match picker.chord.strokes() {
            [first, Stroke::Plain(key)] if *first == base => Some(JumpTarget {
                action_id: picker.name.clone(),
                label: picker.name.clone(),
                directory: picker.description.clone(),
                pick_key: *key,
            }),
            _ => None,
        })
        .collect();
    targets.sort_by_key(|target| target.pick_key);
    targets
}

pub fn launcher_bases(pickers: &[Picker]) -> Vec<Stroke> {
    let mut bases: Vec<Stroke> = Vec::new();
    for picker in pickers {
        if let [first, Stroke::Plain(_)] = picker.chord.strokes()
            && !bases.contains(first)
        {
            bases.push(*first);
        }
    }
    bases
}

pub fn launcher_base(name: &str) -> Option<Stroke> {
    match Chord::parse(name)?.strokes() {
        [base] => Some(*base),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LauncherKeymap;

impl LauncherKeymap {
    pub const LEGEND: Legend = Legend {
        search: "enter search",
        keys: "key opens  enter search  esc close",
    };
}

impl Keymap for LauncherKeymap {
    fn command(&self, key: PickKey, mode: Mode) -> Command {
        match (key, mode) {
            (PickKey::Enter, Mode::Browsing) => Command::Search,
            (PickKey::Character(character), Mode::Browsing) => Command::Key(character),
            _ => ReadlineKeymap.command(key, mode),
        }
    }
}

#[cfg(test)]
mod tests;
