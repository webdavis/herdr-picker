use std::collections::BTreeMap;
use std::path::Path;

use herdr_picker_domain::{
    BUILTIN_PICKERS, BuiltinPicker, Picker, PickerCommand, PickerDeclaration, PickerError, pickers,
};
use serde::Deserialize;

use super::services::{BlueBubblesSettings, UnifiSettings};

mod command_line;

use command_line::{CommandLine, program_words};

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
    title: Option<String>,
    #[serde(default)]
    sources: Vec<String>,
    #[serde(default)]
    steps: BTreeMap<String, StepEntry>,
    pub(super) unifi: Option<UnifiSettings>,
    pub(super) bluebubbles: Option<BlueBubblesSettings>,
}

// A script only a row's next opens: it runs on no key, so it needs nothing but its command.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StepEntry {
    command: CommandLine,
}

fn on() -> bool {
    true
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
            title: None,
            sources: Vec::new(),
            steps: BTreeMap::new(),
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
            (Some(line), None, _) => (
                PickerCommand::Program(program_words(line, pickers_directory, &name)?),
                self.description,
            ),
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
        let steps = self
            .steps
            .into_iter()
            .map(|(step, entry)| {
                let words = program_words(
                    entry.command,
                    pickers_directory,
                    &format!("{name}.steps.{step}"),
                )?;
                Ok((step, words))
            })
            .collect::<Result<_, PickerError>>()?;
        Ok(PickerDeclaration {
            name,
            key: key.unwrap_or_default(),
            description: description.unwrap_or_default(),
            command,
            args: self.args,
            title: self.title,
            sources: self.sources,
            steps,
        })
    }
}
