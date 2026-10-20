use std::collections::BTreeMap;
use std::path::Path;

use herdr_picker_domain::{Picker, PickerCommand, PickerDeclaration, PickerError, pickers};
use herdr_picker_protocol::split_words;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PickerEntry {
    key: String,
    #[serde(default)]
    description: String,
    command: Option<CommandLine>,
    builtin: Option<String>,
    #[serde(default)]
    args: Vec<String>,
}

#[derive(Deserialize)]
#[serde(
    untagged,
    expecting = "a command line, or an array of a program and its arguments"
)]
enum CommandLine {
    Line(String),
    Words(Vec<String>),
}

pub(super) fn declared_pickers(
    entries: BTreeMap<String, PickerEntry>,
    pickers_directory: &Path,
) -> Result<Vec<Picker>, PickerError> {
    let declarations = entries
        .into_iter()
        .map(|(name, entry)| entry.declared(name, pickers_directory))
        .collect::<Result<Vec<_>, _>>()?;
    pickers(declarations)
}

impl PickerEntry {
    fn declared(
        self,
        name: String,
        pickers_directory: &Path,
    ) -> Result<PickerDeclaration, PickerError> {
        let command = match (self.command, self.builtin) {
            (Some(line), None) => {
                let text = line_text(&line);
                program(line, pickers_directory).ok_or(PickerError::UnusableCommand {
                    name: name.clone(),
                    command: text,
                })?
            }
            (None, Some(builtin)) => PickerCommand::Builtin(builtin),
            _ => return Err(PickerError::UnclearCommand { name }),
        };
        Ok(PickerDeclaration {
            name,
            key: self.key,
            description: self.description,
            command,
            args: self.args,
        })
    }
}

// A relative program is one of the scripts in the pickers directory.
fn program(line: CommandLine, pickers_directory: &Path) -> Option<PickerCommand> {
    let mut words = match line {
        CommandLine::Line(text) => split_words(&text).ok()?,
        CommandLine::Words(words) => words,
    };
    let first = words.first_mut().filter(|first| !first.is_empty())?;
    *first = pickers_directory
        .join(&*first)
        .to_string_lossy()
        .into_owned();
    Some(PickerCommand::Program(words))
}

fn line_text(line: &CommandLine) -> String {
    match line {
        CommandLine::Line(text) => text.clone(),
        CommandLine::Words(words) => words.join(" "),
    }
}
