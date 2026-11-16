#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionSeen {
    NoSession,
    OnThisMac { rex_session: String, name: String },
    Elsewhere { name: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnerPlace {
    Front(SessionSeen),
    Background(SessionSeen),
    Closed,
}
