// The engine's own: Ctrl-C closes, Ctrl-N and Ctrl-P move, Ctrl-Q saves the quickfix list, and a
// terminal sends Ctrl-H, Ctrl-I and Ctrl-M as Backspace, Tab and Enter.
const ENGINE_CTRL_LETTERS: &str = "chimnpq";
// Alt-O sends the marked rows to an agent in every picker.
const ENGINE_ALT_LETTERS: &str = "o";

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
    let alt = ('a'..='z')
        .filter(|letter| !ENGINE_ALT_LETTERS.contains(*letter))
        .map(|letter| format!("alt-{letter}"));
    let digits = ('0'..='9').map(|digit| format!("alt-{digit}"));
    // Alt-. shows hidden files, as dotfiles start with a dot; Alt-; opens a picker's small menu.
    let dot = ["alt-.", "alt-;"].map(str::to_string);
    let function = (2..=12).map(|number| format!("f{number}"));
    ["enter".to_string(), "alt-enter".to_string()]
        .into_iter()
        .chain(ctrl)
        .chain(alt)
        .chain(digits)
        .chain(dot)
        .chain(function)
        .map(Key)
}

#[cfg(test)]
mod tests;
