const ENGINE_CTRL_LETTERS: &str = "chimnpq";
const ENGINE_ALT_LETTERS: &str = "o";

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
