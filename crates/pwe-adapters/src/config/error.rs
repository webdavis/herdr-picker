use std::fmt;
use std::path::PathBuf;

use pwe_domain::{PickerError, TargetError};

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
        table: String,
        reading_picker: &'static str,
    },
    UnknownWord {
        path: PathBuf,
        table: &'static str,
        key: &'static str,
        word: String,
        accepted: Vec<&'static str>,
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
            Self::Misplaced {
                path,
                table,
                reading_picker,
            } => (
                path,
                format!("{table} belongs under [pickers.{reading_picker}]"),
            ),
            Self::UnknownWord {
                path,
                table,
                key,
                word,
                accepted,
            } => (
                path,
                format!(
                    "{table} {key} does not accept \"{word}\"; it accepts {}",
                    accepted.join(", ")
                ),
            ),
        };
        write!(
            formatter,
            "cannot read the config {}: {reason}",
            path.display()
        )
    }
}
