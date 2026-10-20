use std::env;
use std::process::Command;

use herdr_picker_adapters::PluginConfig;
use herdr_picker_domain::{Page, PickerCommand};
use herdr_picker_protocol::{PICKER_SOURCE_VARIABLE, PICKER_STATE_VARIABLE, encode_action};

use crate::command::{CommandError, failed};

pub(super) struct StepCommand {
    pub(super) command: Command,
    pub(super) description: String,
    pub(super) source: Option<String>,
    pub(super) sources: Vec<String>,
}

pub(super) fn step_command(
    page: &Page,
    chosen: Option<&str>,
    config: &PluginConfig,
) -> Result<StepCommand, CommandError> {
    let call = match page {
        Page::Picker(call) => call,
        Page::Confirm { question, action } => {
            return Ok(StepCommand {
                command: own(&["builtin", "confirm", question, &encode_action(action)])?,
                description: String::new(),
                source: None,
                sources: Vec::new(),
            });
        }
    };
    let picker = config
        .pickers
        .iter()
        .find(|picker| picker.name == call.name)
        .ok_or_else(|| failed(format!("no picker is named {}", call.name)))?;
    let mut command = match &picker.command {
        PickerCommand::Program(words) => {
            let (program, arguments) = words
                .split_first()
                .ok_or_else(|| failed(format!("the picker {} names no program", picker.name)))?;
            let mut command = Command::new(program);
            command.args(arguments);
            command
        }
        PickerCommand::Builtin(name) => own(&["builtin", name])?,
    };
    command.args(&picker.args).args(&call.args);
    let sources: Vec<String> = picker
        .sources()
        .iter()
        .map(|source| source.to_string())
        .collect();
    let source = chosen
        .map(str::to_string)
        .or_else(|| sources.first().cloned());
    for (variable, value) in [
        (PICKER_STATE_VARIABLE, &call.state),
        (PICKER_SOURCE_VARIABLE, &source),
    ] {
        match value {
            Some(value) => command.env(variable, value),
            None => command.env_remove(variable),
        };
    }
    Ok(StepCommand {
        command,
        description: picker.description.clone(),
        source,
        sources,
    })
}

fn own(arguments: &[&str]) -> Result<Command, CommandError> {
    let mut command = Command::new(env::current_exe().map_err(failed)?);
    command.args(arguments);
    Ok(command)
}
