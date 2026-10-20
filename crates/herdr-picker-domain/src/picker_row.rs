use crate::picker_key::Key;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerRow {
    pub label: String,
    pub preview: Option<String>,
    pub enter: Action,
    // The keys besides enter, each claimed at most once.
    pub keys: Vec<(Key, Action)>,
    pub group: Option<String>,
}

impl PickerRow {
    pub fn action(&self, key: &Key) -> Option<&Action> {
        self.offered()
            .find(|(offered, _)| offered == key)
            .map(|(_, action)| action)
    }

    pub fn offered(&self) -> impl Iterator<Item = (Key, &Action)> {
        std::iter::once((Key::enter(), &self.enter))
            .chain(self.keys.iter().map(|(key, action)| (key.clone(), action)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub path: String,
    pub line: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerCall {
    pub name: String,
    pub args: Vec<String>,
    // Handed to the picker's command as it was given; the command alone reads it.
    pub state: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Insert(String),
    Copy(String),
    CopyStay(String),
    Cd(String),
    Run(String),
    Edit(Vec<Location>),
    Open(String),
    Next(PickerCall),
    Confirm {
        question: String,
        action: Box<Action>,
    },
    Reload,
    Back,
    Nothing,
}
