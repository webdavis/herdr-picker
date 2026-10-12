use crate::JumpTarget;
use crate::field::Edit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Browsing,
    Searching,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Key(char),
    Search,
    Next,
    Previous,
    Accept,
    Cancel,
    Edit(Edit),
    Erase,
    Unbound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pick<'a> {
    Selected(&'a JumpTarget),
    Cancelled,
    Unbound,
    Pending,
}
