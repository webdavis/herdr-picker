// The engine's own: Ctrl-C closes, Ctrl-N and Ctrl-P move, Ctrl-Q saves the quickfix list, and a
// terminal sends Ctrl-H, Ctrl-I and Ctrl-M as Backspace, Tab and Enter.
const ENGINE_CTRL_LETTERS: &str = "chimnpq";

// A key a row may claim: the finder binds every one, and keeps its own meaning where no row claims it.
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
    let ctrl = ('a'..='z')
        .filter(|letter| !ENGINE_CTRL_LETTERS.contains(*letter))
        .map(|letter| format!("ctrl-{letter}"));
    let alt = ('a'..='z').map(|letter| format!("alt-{letter}"));
    // Alt-. shows hidden files, as dotfiles start with a dot.
    let dot = std::iter::once("alt-.".to_string());
    let function = (2..=12).map(|number| format!("f{number}"));
    ["enter".to_string(), "alt-enter".to_string()]
        .into_iter()
        .chain(ctrl)
        .chain(alt)
        .chain(dot)
        .chain(function)
        .map(Key)
}

#[cfg(test)]
mod tests;
