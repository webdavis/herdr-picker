// The engine's own: Ctrl-C closes, Ctrl-N and Ctrl-P move, Ctrl-Q saves the quickfix list, and a
// terminal sends Ctrl-H, Ctrl-I and Ctrl-M as Backspace, Tab and Enter.
const ENGINE_CTRL_LETTERS: &str = "chimnpq";
// Alt-O sends the marked rows to an agent in every picker.
const ENGINE_ALT_LETTERS: &str = "o";

// A key a row may claim: the finder binds every one, and keeps its own meaning where no row claims it.
// Two keys are the same key whatever their labels, which only name what one row does with it.
#[derive(Debug, Clone)]
pub struct Key {
    name: String,
    label: Option<String>,
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Key {}

impl Key {
    pub fn enter() -> Self {
        Self::named("enter")
    }

    fn named(name: &str) -> Self {
        Self {
            name: name.to_string(),
            label: None,
        }
    }

    // A short name for what the key does, which help and the footer show in place of its verb.
    pub fn labelled(self, label: &str) -> Self {
        Self {
            label: Some(label.to_string()),
            ..self
        }
    }

    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    pub fn parse(name: &str) -> Option<Self> {
        // fzf cannot bind Ctrl-Enter; Ghostty sends it as Alt-Enter.
        let name = if name == "ctrl-enter" {
            "alt-enter"
        } else {
            name
        };
        bindable_keys().find(|key| key.name == name)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    // How the key is written for the user.
    pub fn shown(&self) -> &str {
        if self.name == "alt-enter" {
            "ctrl-enter"
        } else {
            &self.name
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
        .map(|name| Key::named(&name))
}

#[cfg(test)]
mod tests;
