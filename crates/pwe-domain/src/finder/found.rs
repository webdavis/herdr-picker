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
pub struct Found<Row = usize> {
    pub query: String,
    pub ending: Ending,
    pub rows: Vec<Row>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Remark {
    Done(String),
    Refused(String),
}
