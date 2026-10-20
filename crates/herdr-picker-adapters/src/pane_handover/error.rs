use std::fmt;

use herdr_picker_application::JumpError;
use herdr_picker_protocol::Refusal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandoverError {
    Unreadable(String),
    Refused(Refusal),
    Herdr(JumpError),
}

impl fmt::Display for HandoverError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable(line) => write!(formatter, "the picker answered {line:?}"),
            Self::Refused(refusal) => write!(formatter, "{refusal}"),
            Self::Herdr(failure) => write!(formatter, "{failure}"),
        }
    }
}

impl From<Refusal> for HandoverError {
    fn from(refusal: Refusal) -> Self {
        Self::Refused(refusal)
    }
}

impl From<JumpError> for HandoverError {
    fn from(failure: JumpError) -> Self {
        Self::Herdr(failure)
    }
}
