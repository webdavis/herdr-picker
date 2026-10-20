#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerRow {
    pub label: String,
    pub preview: Option<String>,
    pub enter: Action,
    pub group: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Insert(String),
    Copy(String),
    Cd(String),
    Run(String),
}
