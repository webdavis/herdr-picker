use std::fmt;
use std::path::PathBuf;

use crate::chord::Chord;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerDeclaration {
    pub name: String,
    pub key: String,
    pub description: String,
    pub command: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picker {
    pub name: String,
    pub chord: Chord,
    pub description: String,
    pub command: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickerError {
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
