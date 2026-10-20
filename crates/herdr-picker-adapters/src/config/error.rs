use std::fmt;
use std::path::PathBuf;

use herdr_picker_domain::{PickerError, TargetError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    Unreadable {
        path: PathBuf,
        reason: String,
    },
    Unparseable {
        path: PathBuf,
        reason: String,
        line: Option<usize>,
    },
    Invalid {
        path: PathBuf,
        cause: TargetError,
    },
    InvalidPicker {
        path: PathBuf,
        cause: PickerError,
    },
    Misplaced {
        path: PathBuf,
        // A table only the network picker reads.
        table: String,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (path, reason) = match self {
            Self::Unreadable { path, reason } => (path, reason.clone()),
            Self::Unparseable { path, reason, line } => (
                path,
                match line {
                    Some(line) => format!("{reason} at line {line}"),
                    None => reason.clone(),
                },
            ),
            Self::Invalid { path, cause } => (path, cause.to_string()),
            Self::InvalidPicker { path, cause } => (path, cause.to_string()),
            Self::Misplaced { path, table } => {
                (path, format!("{table} belongs under [pickers.network]"))
            }
        };
        write!(
            formatter,
            "cannot read the config {}: {reason}",
            path.display()
        )
    }
}
