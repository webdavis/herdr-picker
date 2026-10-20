use std::path::Path;

use herdr_picker_adapters::{
    Chrome, Helper, Picked, Placement, PluginConfig, SWITCH_SOURCE, fzf_arguments, read_view,
    remove_run_files, run_picker, write_view,
};
use herdr_picker_domain::{Action, Key, Page, Spot, Trail, View, ViewState, act, header};

use crate::command::{CommandError, failed};

mod command;

use command::{StepCommand, step_command};

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
        let StepCommand {
            mut command,
            description,
            source,
            sources,
        } = step_command(&step.page, step.spot.source.as_deref(), self.config)?;
        let titles = trail.titles();
        let first_view = View::Streamed {
            group: step.spot.group.clone(),
        };
        let header = header(&titles, &first_view, source.as_deref(), &[]);
        remove_run_files(self.cache);
        let state = ViewState {
            view: first_view,
            titles,
            description: description.clone(),
            source,
            sources,
        };
        write_view(self.cache, &state).map_err(failed)?;
        let chrome = Chrome {
            helper: self.helper,
            header: &header,
            description: &description,
            query: &step.spot.query,
        };
        let arguments = fzf_arguments(self.config.theme, &chrome, self.placement);
        let picked = run_picker(&mut command, Path::new("fzf"), &arguments, self.cache);
        let left = read_view(self.cache);
        remove_run_files(self.cache);
        let name = match &step.page {
            Page::Picker(call) => call.name.as_str(),
            Page::Confirm { .. } => "confirm",
        };
        let picked = picked.map_err(|failure| failed(format!("picker {name}: {failure}")))?;
        Ok(picked.map(|picked| {
            let action = chosen_action(&picked, left.source.as_deref());
            // The query a switch leaves behind filtered the chooser, not the rows.
            let switched = picked.key == SWITCH_SOURCE;
            let spot = Spot {
                query: if switched {
                    String::new()
                } else {
                    picked.query
                },
                group: match left.view {
                    View::Streamed { group } | View::List { group } | View::Chooser { group } => {
                        group
                    }
                },
                source: left.source,
            };
            (action, spot)
        }))
    }
}

fn chosen_action(picked: &Picked, source: Option<&str>) -> Action {
    match picked.key.as_str() {
        "esc" => return Action::Back,
        SWITCH_SOURCE => {
            return source.map_or(Action::Nothing, |source| Action::Source(source.to_string()));
        }
        _ => {}
    }
    let rows: Vec<_> = picked.rows.iter().collect();
    Key::parse(&picked.key)
        .and_then(|key| act(&rows, &key).ok())
        .unwrap_or(Action::Nothing)
}

#[cfg(test)]
mod tests;
