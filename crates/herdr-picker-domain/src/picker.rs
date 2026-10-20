use std::fmt;

use crate::chord::Chord;
use crate::picker_builtin::BUILTIN_PICKERS;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerDeclaration {
    pub name: String,
    pub key: String,
    pub description: String,
    pub command: PickerCommand,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickerCommand {
    // The program, then its own arguments.
    Program(Vec<String>),
    // One of herdr-picker's own pickers, by name.
    Builtin(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picker {
    pub name: String,
    pub chord: Chord,
    pub description: String,
    pub command: PickerCommand,
    pub args: Vec<String>,
}

impl Picker {
    pub fn sources(&self) -> &'static [&'static str] {
        let PickerCommand::Builtin(name) = &self.command else {
            return &[];
        };
        BUILTIN_PICKERS
            .iter()
            .find(|builtin| builtin.name == name)
            .map_or(&[], |builtin| builtin.sources)
    }
}

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
                "the picker {name} declares the key {key:?}, which is not a chord: ctrl-<letter> or alt-<letter>, then any plain letters, like \"ctrl-t b\""
            ),
            Self::CollidingKey { first, second, key } => write!(
                formatter,
                "the pickers {first} and {second} both run on {key}"
            ),
        }
    }
}

pub fn pickers(
    declarations: impl IntoIterator<Item = PickerDeclaration>,
) -> Result<Vec<Picker>, PickerError> {
    let mut accepted: Vec<Picker> = Vec::new();
    for declaration in declarations {
        let picker = picker(declaration.clone())?;
        if let Some(first) = accepted.iter().find(|other| other.chord == picker.chord) {
            return Err(PickerError::CollidingKey {
                first: first.name.clone(),
                second: declaration.name,
                key: declaration.key,
            });
        }
        accepted.push(picker);
    }
    Ok(accepted)
}

fn picker(declaration: PickerDeclaration) -> Result<Picker, PickerError> {
    let PickerDeclaration {
        name,
        key,
        description,
        command,
        args,
    } = declaration;
    if !is_bare_word(&name) {
        return Err(PickerError::UnusableName { name });
    }
    let Some(chord) = Chord::parse(&key) else {
        return Err(PickerError::UnusableKey { name, key });
    };
    Ok(Picker {
        name,
        chord,
        description,
        command,
        args,
    })
}

fn is_bare_word(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
}

#[cfg(test)]
mod tests;
