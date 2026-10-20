use std::env;
use std::path::Path;
use std::process::Command;

use herdr_picker_adapters::{
    Chrome, Helper, Picked, Placement, PluginConfig, fzf_arguments, read_view, remove_run_files,
    run_picker, write_view,
};
use herdr_picker_domain::{
    Action, Key, PickerCommand, Source, Spot, Trail, View, ViewState, act, header,
};
use herdr_picker_protocol::{PICKER_STATE_VARIABLE, encode_action};

use crate::command::{CommandError, failed};

pub(crate) struct Run<'a> {
    pub(crate) config: &'a PluginConfig,
    pub(crate) helper: Helper<'a>,
    pub(crate) cache: &'a Path,
    pub(crate) placement: Placement,
}

impl Run<'_> {
    // The action the finder closed on and where the user was, or nothing when it was closed.
    pub(crate) fn show(&self, trail: &Trail) -> Result<Option<(Action, Spot)>, CommandError> {
        let step = trail.current();
        let (mut command, description) = step_command(&step.source, self.config)?;
        let titles = trail.titles();
        let first_view = View::Streamed {
            group: step.spot.group.clone(),
        };
        let header = header(&titles, &first_view, &[]);
        remove_run_files(self.cache);
        write_view(
            self.cache,
            &ViewState {
                view: first_view,
                titles,
                description: description.clone(),
            },
        )
        .map_err(failed)?;
        let chrome = Chrome {
            helper: self.helper,
            header: &header,
            description: &description,
            query: &step.spot.query,
        };
        let arguments = fzf_arguments(self.config.theme, &chrome, self.placement);
        let picked = run_picker(&mut command, Path::new("fzf"), &arguments, self.cache);
        let group = match read_view(self.cache).view {
            View::Streamed { group } | View::List { group } | View::Chooser { group } => group,
        };
        remove_run_files(self.cache);
        let name = match &step.source {
            Source::Picker(call) => call.name.as_str(),
            Source::Confirm { .. } => "confirm",
        };
        let picked = picked.map_err(|failure| failed(format!("picker {name}: {failure}")))?;
        Ok(picked.map(|picked| {
            let action = chosen_action(&picked);
            let spot = Spot {
                query: picked.query,
                group,
            };
            (action, spot)
        }))
    }
}

fn chosen_action(picked: &Picked) -> Action {
    if picked.key == "esc" {
        return Action::Back;
    }
    let rows: Vec<_> = picked.rows.iter().collect();
    Key::parse(&picked.key)
        .and_then(|key| act(&rows, &key).ok())
        .unwrap_or(Action::Nothing)
}

fn step_command(source: &Source, config: &PluginConfig) -> Result<(Command, String), CommandError> {
    let call = match source {
        Source::Picker(call) => call,
        Source::Confirm { question, action } => {
            let action = encode_action(action);
            return Ok((
                own(&["builtin", "confirm", question, &action])?,
                String::new(),
            ));
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
    match &call.state {
        Some(state) => command.env(PICKER_STATE_VARIABLE, state),
        None => command.env_remove(PICKER_STATE_VARIABLE),
    };
    Ok((command, picker.description.clone()))
}

fn own(arguments: &[&str]) -> Result<Command, CommandError> {
    let mut command = Command::new(env::current_exe().map_err(failed)?);
    command.args(arguments);
    Ok(command)
}

#[cfg(test)]
mod tests;
