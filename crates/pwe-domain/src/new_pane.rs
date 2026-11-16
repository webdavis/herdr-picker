use crate::picker_key::Key;
use crate::picker_row::Action;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewPane {
    Left,
    Below,
    Above,
    Right,
    Tab,
}

const OPENINGS: [(&str, &str, NewPane); 5] = [
    ("alt-h", "left", NewPane::Left),
    ("alt-j", "below", NewPane::Below),
    ("alt-k", "above", NewPane::Above),
    ("alt-l", "right", NewPane::Right),
    ("alt-n", "tab", NewPane::Tab),
];

impl NewPane {
    pub fn of(key: &Key) -> Option<Self> {
        OPENINGS
            .iter()
            .find(|(name, _, _)| *name == key.name())
            .map(|(_, _, pane)| *pane)
    }

    pub fn word(self) -> &'static str {
        OPENINGS
            .iter()
            .find(|(_, _, pane)| *pane == self)
            .map_or("", |(_, word, _)| word)
    }

    pub(crate) fn keys() -> impl Iterator<Item = (Key, Self)> {
        OPENINGS
            .iter()
            .filter_map(|(name, _, pane)| Some((Key::parse(name)?, *pane)))
    }

    pub(crate) fn takes(action: &Action) -> bool {
        matches!(action, Action::Run(_) | Action::Edit(_) | Action::Cd(_))
    }

    pub fn parse(word: &str) -> Option<Self> {
        OPENINGS
            .iter()
            .find(|(_, known, _)| *known == word)
            .map(|(_, _, pane)| *pane)
    }
}

#[cfg(test)]
mod tests;
