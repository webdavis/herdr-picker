use std::collections::BTreeMap;
use std::path::Path;

use herdr_picker_domain::{
    BUILTIN_PICKERS, BuiltinPicker, Picker, PickerCommand, PickerDeclaration, PickerError, pickers,
};
use herdr_picker_protocol::split_words;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PickerEntry {
    key: Option<String>,
    description: Option<String>,
    command: Option<CommandLine>,
    builtin: Option<String>,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default = "on")]
    enabled: bool,
}

fn on() -> bool {
    true
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

// The configured pickers, then each built-in one the config leaves alone, unless a configured
// picker already runs on its key.
pub(super) fn declared_pickers(
    entries: BTreeMap<String, PickerEntry>,
    pickers_directory: &Path,
) -> Result<Vec<Picker>, PickerError> {
    let untouched = BUILTIN_PICKERS
        .iter()
        .filter(|builtin| !entries.contains_key(builtin.name))
        .map(|builtin| PickerDeclaration {
            name: builtin.name.to_string(),
            key: builtin.key.to_string(),
            description: builtin.description.to_string(),
            command: PickerCommand::Builtin(builtin.name.to_string()),
            args: Vec::new(),
        })
        .collect::<Vec<_>>();
    let declarations = entries
        .into_iter()
        .filter(|(_, entry)| entry.enabled)
        .map(|(name, entry)| entry.declared(name, pickers_directory))
        .collect::<Result<Vec<_>, _>>()?;
    let mut accepted = pickers(declarations)?;
    for builtin in pickers(untouched)? {
        if accepted.iter().all(|picker| picker.chord != builtin.chord) {
            accepted.push(builtin);
        }
    }
    Ok(accepted)
}

impl PickerEntry {
    // A table named for a built-in picker that names no command changes that built-in picker.
    fn declared(
        self,
        name: String,
        pickers_directory: &Path,
    ) -> Result<PickerDeclaration, PickerError> {
        let shipped: Option<&BuiltinPicker> =
            BUILTIN_PICKERS.iter().find(|builtin| builtin.name == name);
        let (command, description) = match (self.command, self.builtin, shipped) {
            (Some(line), None, _) => {
                let text = line_text(&line);
                let command =
                    program(line, pickers_directory).ok_or(PickerError::UnusableCommand {
                        name: name.clone(),
                        command: text,
                    })?;
                (command, self.description)
            }
            (None, Some(builtin), _) => (PickerCommand::Builtin(builtin), self.description),
            (None, None, Some(shipped)) => (
                PickerCommand::Builtin(shipped.name.to_string()),
                self.description
                    .or_else(|| Some(shipped.description.to_string())),
            ),
            _ => return Err(PickerError::UnclearCommand { name }),
        };
        let key = self
            .key
            .or_else(|| shipped.map(|shipped| shipped.key.to_string()));
        Ok(PickerDeclaration {
            name,
            key: key.unwrap_or_default(),
            description: description.unwrap_or_default(),
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
