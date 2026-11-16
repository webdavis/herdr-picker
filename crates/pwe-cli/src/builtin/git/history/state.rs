// Alt-S flips the list between newest and oldest first; the step keeps it in its state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Sort {
    NewestFirst,
    OldestFirst,
}

impl Sort {
    pub(super) fn from_state(state: Option<&str>) -> Self {
        let read: Option<serde_json::Value> =
            state.and_then(|state| serde_json::from_str(state).ok());
        match read.and_then(|state| state["sort"].as_bool()) {
            Some(true) => Self::OldestFirst,
            _ => Self::NewestFirst,
        }
    }

    pub(super) fn flipped(self) -> Self {
        match self {
            Self::NewestFirst => Self::OldestFirst,
            Self::OldestFirst => Self::NewestFirst,
        }
    }

    pub(super) fn state(self) -> String {
        match self {
            Self::NewestFirst => "{}".to_string(),
            Self::OldestFirst => "{\"sort\":true}".to_string(),
        }
    }
}
