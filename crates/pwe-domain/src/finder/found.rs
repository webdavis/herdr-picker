use crate::Key;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ending {
    Key(Key),
    Back,
    SendRows,
    SaveQuickfix,
    SwitchSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    pub query: String,
    pub ending: Ending,
    pub rows: Vec<usize>,
}
