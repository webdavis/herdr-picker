use std::env;
use std::process::Command;

use herdr_picker_adapters::PluginConfig;
use herdr_picker_domain::{
    Page, PickerCommand, PickerStep, follows_query, is_step_builtin, step_sources, title,
};
use herdr_picker_protocol::{PICKER_SOURCE_VARIABLE, PICKER_STATE_VARIABLE, encode_action};

use crate::command::{CommandError, failed};

pub(super) struct StepCommand {
    pub(super) command: Command,
    pub(super) description: String,
    pub(super) source: Option<String>,
    pub(super) sources: Vec<String>,
    // Its rows follow the query, so it runs again as the user types.
    pub(super) live: bool,
}

// `trail` names the pickers the user came through, whose steps a next may name.
pub(super) fn step_command(
    page: &Page,
    chosen: Option<&str>,
    config: &PluginConfig,
    trail: &[&str],
) -> Result<StepCommand, CommandError> {
    let call = match page {
        Page::Picker(call) => call,
        Page::Confirm { question, action } => {
            return Ok(StepCommand {
                command: own(&["builtin", "confirm", question, &encode_action(action)])?,
                description: String::new(),
                source: None,
                sources: Vec::new(),
                live: false,
            });
        }
    };
    if let Some(step) = own_step(&call.name, config, trail) {
        let mut command = program(&step.command, &call.name)?;
        command.args(&call.args);
        let source = chosen_or_first(chosen, &step.sources);
        with_variables(&mut command, &call.state, &source);
        return Ok(StepCommand {
            command,
            description: String::new(),
            source,
            sources: step.sources.clone(),
            live: step.live,
        });
    }
    if is_step_builtin(&call.name) && config.pickers.iter().all(|picker| picker.name != call.name) {
        let mut command = own(&["builtin", &call.name])?;
        command.args(&call.args);
        let sources = step_sources(&call.name);
        let source = chosen.or(sources.first().copied()).map(str::to_string);
        with_variables(&mut command, &call.state, &source);
        return Ok(StepCommand {
            command,
            description: String::new(),
            live: follows_query(&call.name, source.as_deref()),
            source,
            sources: sources.iter().map(|source| source.to_string()).collect(),
        });
    }
    let picker = config
        .pickers
        .iter()
        .find(|picker| picker.name == call.name)
        .ok_or_else(|| failed(format!("no picker is named {}", call.name)))?;
    let mut command = match &picker.command {
        PickerCommand::Program(words) => program(words, &picker.name)?,
        PickerCommand::Builtin(name) => own(&["builtin", name])?,
    };
    command.args(&picker.args).args(&call.args);
    let sources = picker.sources.clone();
    let source = chosen_or_first(chosen, &sources);
    with_variables(&mut command, &call.state, &source);
    let live = match &picker.command {
        PickerCommand::Builtin(name) => follows_query(name, source.as_deref()),
        PickerCommand::Program(_) => picker.live,
    };
    Ok(StepCommand {
        command,
        description: picker.description.clone(),
        source,
        sources,
        live,
    })
}

fn own_step<'a>(name: &str, config: &'a PluginConfig, trail: &[&str]) -> Option<&'a PickerStep> {
    trail.iter().rev().find_map(|owner| {
        let picker = config.pickers.iter().find(|picker| picker.name == *owner)?;
        picker.steps.get(name)
    })
}

// A source the step does not offer, such as one carried from another step, falls back to its first.
fn chosen_or_first(chosen: Option<&str>, sources: &[String]) -> Option<String> {
    let offered = |chosen: &&str| sources.iter().any(|source| source == chosen);
    chosen
        .filter(offered)
        .map(str::to_string)
        .or_else(|| sources.first().cloned())
}

pub(super) fn picker_title(name: &str, config: &PluginConfig) -> String {
    config
        .pickers
        .iter()
        .find(|picker| picker.name == name)
        .map_or_else(|| title(name), |picker| picker.title.clone())
}

fn program(words: &[String], name: &str) -> Result<Command, CommandError> {
    let (program, arguments) = words
        .split_first()
        .ok_or_else(|| failed(format!("the picker {name} names no program")))?;
    let mut command = Command::new(program);
    command.args(arguments);
    Ok(command)
}

fn with_variables(command: &mut Command, state: &Option<String>, source: &Option<String>) {
    for (variable, value) in [
        (PICKER_STATE_VARIABLE, state),
        (PICKER_SOURCE_VARIABLE, source),
    ] {
        match value {
            Some(value) => command.env(variable, value),
            None => command.env_remove(variable),
        };
    }
}

fn own(arguments: &[&str]) -> Result<Command, CommandError> {
    let mut command = Command::new(env::current_exe().map_err(failed)?);
    command.args(arguments);
    Ok(command)
}
