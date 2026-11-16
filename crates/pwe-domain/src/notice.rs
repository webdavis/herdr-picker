#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    NeedsYou,
    Finished,
    Quiet,
}

impl Level {
    pub fn word(self) -> &'static str {
        match self {
            Self::NeedsYou => "needs-you",
            Self::Finished => "finished",
            Self::Quiet => "quiet",
        }
    }

    pub fn parse(word: &str) -> Option<Self> {
        [Self::NeedsYou, Self::Finished, Self::Quiet]
            .into_iter()
            .find(|level| level.word() == word)
    }

    pub(crate) fn urgency(self) -> u8 {
        match self {
            Self::NeedsYou => 2,
            Self::Finished => 1,
            Self::Quiet => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pane {
    pub rex_session: String,
    pub tab: String,
    pub block: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    pub id: String,
    pub key: String,
    pub source: String,
    pub kind: String,
    pub level: Level,
    pub text: String,
    pub pane: Option<Pane>,
    pub tab_label: Option<String>,
    pub raised_at: u64,
    pub updated_at: u64,
    pub resolved_at: Option<u64>,
    pub read_at: Option<u64>,
}

impl Notice {
    pub fn is_unread(&self) -> bool {
        self.read_at.is_none()
    }

    pub(crate) fn is_about(&self, pane: &Pane) -> bool {
        self.pane.as_ref() == Some(pane)
    }
}
