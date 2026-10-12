use crate::JumpTarget;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame<'a> {
    pub rows: Vec<Row<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row<'a> {
    pub target: &'a JumpTarget,
    pub selected: bool,
}
