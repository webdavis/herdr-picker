use crate::notice::{Level, Notice, Pane};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Raised {
    pub key: String,
    pub source: String,
    pub kind: String,
    pub level: Level,
    pub text: String,
    pub pane: Option<Pane>,
    pub tab_label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RexFocus {
    Front(Pane),
    Background,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoticeInput {
    Raise(Raised),
    Resolve { key: String },
    OwnerMoved(RexFocus),
    TimePassed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoticeEffect {
    Show {
        float: String,
        slot: usize,
        level: Level,
        line: String,
        seconds: u64,
    },
    Relabel {
        float: String,
        line: String,
        seconds: u64,
    },
    Close {
        float: String,
    },
    SaveRow(Box<Notice>),
}
