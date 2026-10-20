use crate::new_pane::NewPane;
use crate::picker_key::Key;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerRow {
    pub label: String,
    pub preview: Option<Preview>,
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
pub enum Preview {
    Text(String),
    // Drawn only when the row is highlighted, so a long list never waits on its previews.
    Path(String),
    // A program and its arguments, run only when the row is highlighted; its output is the preview.
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
    // Handed to the picker's command as it was given; the command alone reads it.
    pub state: Option<String>,
    // Typed into the step's search line when it opens.
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
    // Shows the same step again, its rows read from the named source.
    Source(String),
    // Shows the same step again, its command handed this state in place of the one it had.
    State(String),
    Reload,
    Back,
    Nothing,
    // A run, edit or cd row's own action, opened in a new pane rather than the one underneath.
    InPane {
        pane: NewPane,
        action: Box<Action>,
    },
}
