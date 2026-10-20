use std::fmt;

use crate::keymap::SEARCH_CHARACTER;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetError {
    NoWorkspaces,
    EmptyLabel,
    EmptyDirectory {
        label: String,
    },
    CollidingActionId {
        first: String,
        second: String,
        action_id: String,
    },
    UnusablePickKey {
        label: String,
        key: String,
    },
    LabelHasNoDefaultPickKey {
        label: String,
    },
    CollidingPickKey {
        first: String,
        second: String,
        key: char,
    },
}

impl fmt::Display for TargetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoWorkspaces => write!(formatter, "the [workspaces] table declares no workspace"),
            Self::EmptyLabel => write!(formatter, "a workspace label is empty"),
            Self::EmptyDirectory { label } => {
                write!(formatter, "the workspace {label} has an empty directory")
            }
            Self::CollidingActionId {
                first,
                second,
                action_id,
            } => write!(
                formatter,
                "the labels {first} and {second} both derive the action id {action_id}"
            ),
            Self::UnusablePickKey { label, key } => write!(
                formatter,
                "the workspace {label} declares the pick key {key:?}, which is not one printable ASCII character other than {SEARCH_CHARACTER}, the key that starts a search"
            ),
            Self::LabelHasNoDefaultPickKey { label } => write!(
                formatter,
                "the workspace {label} starts with a character that cannot be a pick key, so it has to declare one"
            ),
            Self::CollidingPickKey { first, second, key } => write!(
                formatter,
                "the workspaces {first} and {second} both pick on {key}"
            ),
        }
    }
}
