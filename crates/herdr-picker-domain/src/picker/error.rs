use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickerError {
    UnclearCommand {
        name: String,
    },
    UnusableCommand {
        name: String,
        command: String,
    },
    UnusableName {
        name: String,
    },
    UnusableKey {
        name: String,
        key: String,
    },
    CollidingKey {
        first: String,
        second: String,
        key: String,
    },
    UnusableSource {
        name: String,
        source: String,
    },
    BuiltinSources {
        name: String,
    },
    BuiltinLive {
        name: String,
    },
}

impl fmt::Display for PickerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnclearCommand { name } => write!(
                formatter,
                "the picker {name} needs exactly one of command or builtin"
            ),
            Self::UnusableCommand { name, command } => write!(
                formatter,
                "the picker {name} runs {command:?}, which is not a program then its arguments"
            ),
            Self::UnusableName { name } => write!(
                formatter,
                "the picker name {name:?} is not only letters, digits, - and _"
            ),
            Self::UnusableKey { name, key } => write!(
                formatter,
                "the picker {name} declares the key {key:?}, which is not a chord: ctrl-<letter> or alt-<letter>, then any plain letters or punctuation, like \"ctrl-t b\""
            ),
            Self::CollidingKey { first, second, key } => write!(
                formatter,
                "the pickers {first} and {second} both run on {key}"
            ),
            Self::UnusableSource { name, source } => write!(
                formatter,
                "the picker {name} offers the source {source:?}, which is not lowercase text"
            ),
            Self::BuiltinSources { name } => write!(
                formatter,
                "the picker {name} runs a built-in picker, which reads its own sources"
            ),
            Self::BuiltinLive { name } => write!(
                formatter,
                "the picker {name} runs a built-in picker, which decides itself whether it follows the query"
            ),
        }
    }
}
