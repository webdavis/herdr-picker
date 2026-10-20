const CTRL_LETTERS: &str = "orstvxy";
const EDITING_ALT_LETTERS: &str = "bdf";

// A key fzf leaves unbound by default, so a row claiming it never steals typing or editing.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key(String);

impl Key {
    pub fn enter() -> Self {
        Self("enter".to_string())
    }

    pub fn parse(name: &str) -> Option<Self> {
        // fzf cannot bind Ctrl-Enter; Ghostty sends it as Alt-Enter.
        let name = if name == "ctrl-enter" {
            "alt-enter"
        } else {
            name
        };
        bindable_keys().find(|key| key.0 == name)
    }

    pub fn name(&self) -> &str {
        &self.0
    }

    pub fn label(&self) -> &str {
        if self.0 == "alt-enter" {
            "ctrl-enter"
        } else {
            &self.0
        }
    }
}

pub fn bindable_keys() -> impl Iterator<Item = Key> {
    let ctrl = CTRL_LETTERS.chars().map(|letter| format!("ctrl-{letter}"));
    let alt = ('a'..='z')
        .filter(|letter| !EDITING_ALT_LETTERS.contains(*letter))
        .map(|letter| format!("alt-{letter}"));
    let function = (2..=12).map(|number| format!("f{number}"));
    ["enter".to_string(), "alt-enter".to_string()]
        .into_iter()
        .chain(ctrl)
        .chain(alt)
        .chain(function)
        .map(Key)
}

#[cfg(test)]
mod tests;
