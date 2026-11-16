use crate::new_pane::NewPane;
use crate::picker_key::Key;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerRow {
    pub label: String,
    pub preview: Option<Preview>,
    pub enter: Action,
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
pub enum Preview {
    Text(String),
    Path(String),
    Command(Vec<String>),
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
    pub state: Option<String>,
    pub query: Option<String>,
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
    Source(String),
    State(String),
    Reload,
    Back,
    Nothing,
    InPane {
        pane: NewPane,
        action: Box<Action>,
    },
    Herdr(HerdrCall),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HerdrCall {
    FocusPane(String),
    FocusTab(String),
    FocusWorkspace(String),
    CloseTab(String),
    Prompt { pane_id: String, text: String },
}
