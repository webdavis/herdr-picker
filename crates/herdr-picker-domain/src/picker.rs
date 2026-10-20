use std::collections::BTreeMap;

use crate::chord::Chord;
use crate::picker_builtin::BUILTIN_PICKERS;
use crate::picker_header::title;

mod error;
mod step;
mod words;

pub use error::PickerError;
pub use step::PickerStep;
use words::{checked_sources, is_bare_word};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerDeclaration {
    pub name: String,
    pub key: String,
    pub description: String,
    pub command: PickerCommand,
    pub args: Vec<String>,
    // The header's title in place of the name's.
    pub title: Option<String>,
    // The first is the one the picker opens on; a built-in picker reads its own.
    pub sources: Vec<String>,
    // The scripts its rows' next opens, by name.
    pub steps: BTreeMap<String, PickerStep>,
    // A scripted picker's rows follow the query; a built-in picker decides that itself.
    pub live: bool,
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
    pub title: String,
    pub chord: Chord,
    pub description: String,
    pub command: PickerCommand,
    pub args: Vec<String>,
    pub sources: Vec<String>,
    pub steps: BTreeMap<String, PickerStep>,
    pub live: bool,
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
        title: declared_title,
        sources,
        steps,
        live,
    } = declaration;
    if !is_bare_word(&name) {
        return Err(PickerError::UnusableName { name });
    }
    let Some(chord) = Chord::parse(&key) else {
        return Err(PickerError::UnusableKey { name, key });
    };
    checked_sources(&name, &sources)?;
    for (step, declared) in &steps {
        checked_sources(&format!("{name}.steps.{step}"), &declared.sources)?;
    }
    let sources = match &command {
        PickerCommand::Builtin(_) if !sources.is_empty() => {
            return Err(PickerError::BuiltinSources { name });
        }
        PickerCommand::Builtin(_) if live => return Err(PickerError::BuiltinLive { name }),
        PickerCommand::Builtin(builtin) => builtin_sources(builtin),
        PickerCommand::Program(_) => sources,
    };
    Ok(Picker {
        title: declared_title.unwrap_or_else(|| title(&name)),
        name,
        chord,
        description,
        command,
        args,
        sources,
        steps,
        live,
    })
}

fn builtin_sources(name: &str) -> Vec<String> {
    BUILTIN_PICKERS
        .iter()
        .find(|builtin| builtin.name == name)
        .map_or(Vec::new(), |builtin| {
            builtin
                .sources
                .iter()
                .map(|source| source.to_string())
                .collect()
        })
}

#[cfg(test)]
mod tests;
