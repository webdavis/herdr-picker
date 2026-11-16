use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunnerFileError {
    Unreadable {
        file: String,
        reason: String,
    },
    Missing {
        file: String,
        field: &'static str,
    },
    Empty {
        file: String,
        field: &'static str,
    },
    NotOneOf {
        file: String,
        field: &'static str,
        value: String,
        accepted: &'static [&'static str],
    },
    NotKebabCase {
        file: String,
        field: &'static str,
        value: String,
    },
}

impl fmt::Display for RunnerFileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunnerFileError::Unreadable { file, reason } => write!(formatter, "{file}: {reason}"),
            RunnerFileError::Missing { file, field } => {
                write!(formatter, "{file}: field `{field}` is missing")
            }
            RunnerFileError::Empty { file, field } => {
                write!(formatter, "{file}: field `{field}` is empty")
            }
            RunnerFileError::NotOneOf {
                file,
                field,
                value,
                accepted,
            } => write!(
                formatter,
                "{file}: field `{field}` is `{value}`; it takes one of: {}",
                accepted.join(", ")
            ),
            RunnerFileError::NotKebabCase { file, field, value } => write!(
                formatter,
                "{file}: field `{field}` is `{value}`; it takes lowercase letters, digits and dashes"
            ),
        }
    }
}

impl std::error::Error for RunnerFileError {}
