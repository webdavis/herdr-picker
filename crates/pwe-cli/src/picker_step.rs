use std::path::Path;

use pwe_adapters::{PluginConfig, open_quickfix};
use pwe_application::{StepFinder, StepOutcome, StepSetup};
use pwe_domain::finder::{Ending, Found};
use pwe_domain::{Action, Page, PickerRow, Spot, Trail, View, ViewState, act, header};

use crate::builtin::agents::send_call;
use crate::command::{CommandError, failed};

mod command;

use command::{StepCommand, picker_title, step_command};

pub(crate) struct Run<'a, F> {
    pub(crate) config: &'a PluginConfig,
    pub(crate) finder: &'a F,
    pub(crate) quickfix: &'a Path,
}

impl<F: StepFinder> Run<'_, F> {
    pub(crate) fn show(&self, trail: &Trail) -> Result<Option<(Action, Spot)>, CommandError> {
        let step = trail.current();
        let StepCommand {
            command,
            description,
            source,
            sources,
            live,
        } = step_command(
            &step.page,
            step.spot.source.as_deref(),
            self.config,
            &trail.names(),
        )?;
        let titles = trail.titles(|name| picker_title(name, self.config));
        let first_picker_title = titles.first().cloned().unwrap_or_default();
        let first_view = View::Streamed {
            group: step.spot.group.clone(),
        };
        let header = header(&titles, &first_view, source.as_deref(), &[]);
        let setup = StepSetup {
            rows: command,
            query: step.spot.query.clone(),
            header,
            view: ViewState {
                view: first_view,
                titles,
                description,
                source,
                sources,
            },
            live,
        };
        let name = match &step.page {
            Page::Picker(call) => call.name.as_str(),
            Page::Confirm { .. } => "confirm",
        };
        let outcome = self
            .finder
            .find(setup)
            .map_err(|failure| failed(format!("picker {name}: {failure}")))?;
        Ok(outcome.map(|StepOutcome { found, view_left }| {
            let action = chosen_action(
                &found,
                view_left.source.as_deref(),
                self.quickfix,
                &first_picker_title,
            );
            let query_filtered_the_chooser = found.ending == Ending::SwitchSource;
            let spot = Spot {
                query: if query_filtered_the_chooser {
                    String::new()
                } else {
                    found.query
                },
                group: match view_left.view {
                    View::Streamed { group } | View::List { group } | View::Chooser { group } => {
                        group
                    }
                },
                source: view_left.source,
            };
            (action, spot)
        }))
    }
}

fn chosen_action(
    found: &Found<PickerRow>,
    source: Option<&str>,
    quickfix: &Path,
    first_picker_title: &str,
) -> Action {
    match &found.ending {
        Ending::Back => Action::Back,
        Ending::SendRows => Action::Next(send_call(&found.rows, first_picker_title)),
        Ending::SaveQuickfix => Action::Run(open_quickfix(quickfix, None)),
        Ending::SwitchSource => {
            source.map_or(Action::Nothing, |source| Action::Source(source.to_string()))
        }
        Ending::Key(key) => {
            let rows: Vec<_> = found.rows.iter().collect();
            act(&rows, key).unwrap_or(Action::Nothing)
        }
    }
}

#[cfg(test)]
mod tests;
