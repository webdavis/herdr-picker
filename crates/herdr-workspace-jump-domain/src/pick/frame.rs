use crate::JumpTarget;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame<'a> {
    pub search: Option<SearchLine>,
    pub rows: Vec<Row<'a>>,
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchLine {
    pub text: String,
    pub cursor: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row<'a> {
    pub target: &'a JumpTarget,
    pub selected: bool,
    pub current: bool,
}
