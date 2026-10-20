use std::fmt;
use std::path::PathBuf;

use herdr_picker_protocol::RowError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickerRunError {
    CannotStart { program: PathBuf, reason: String },
    CannotCache { path: PathBuf, reason: String },
    BadRow(RowError),
    CommandFailed(String),
    FinderFailed(String),
}

impl fmt::Display for PickerRunError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CannotStart { program, reason } => {
                write!(formatter, "cannot start {}: {reason}", program.display())
            }
            Self::CannotCache { path, reason } => {
                write!(
                    formatter,
                    "cannot cache the rows in {}: {reason}",
                    path.display()
                )
            }
            Self::BadRow(cause) => write!(formatter, "{cause}"),
            Self::CommandFailed(status) => {
                write!(formatter, "the picker command stopped with {status}")
            }
            Self::FinderFailed(reason) => write!(formatter, "the finder failed: {reason}"),
        }
    }
}
