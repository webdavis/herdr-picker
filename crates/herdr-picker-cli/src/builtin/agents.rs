use std::io::Write;

use herdr_picker_domain::{Action, PickerRow};

use super::page::notice;
use super::rows::{RowWriter, key};
use super::{Asked, Places, written};
use crate::command::CommandError;

mod agent_row;
mod branch;
mod herdr_view;
mod instructions;
mod running;
mod send;
mod sessions;
mod start;

use herdr_view::herdr_view;
pub(crate) use send::send_call;

// Ctrl-R asks herdr again rather than reading what the status hook saved.
const FRESH: &str = r#"{"fresh":true}"#;

// The agents picker, by source, and the steps Alt-O opens in any picker.
pub(super) fn agents(
    name: &str,
    asked: &Asked,
    places: &Places,
    rows: &mut RowWriter<impl Write>,
) -> Result<(), CommandError> {
    let listed = match (name, asked.source.unwrap_or_default()) {
        ("send-to", _) => send::send_to(asked, places),
        ("quick-answer", _) => send::quick_answer(asked),
        ("answer", _) => send::answer(asked),
        (_, source) => sources(source, asked, places),
    };
    written(listed.iter().try_for_each(|row| rows.row(row)))
}

fn sources(source: &str, asked: &Asked, places: &Places) -> Vec<PickerRow> {
    let listed = match source {
        "instructions" => instructions::instructions(places),
        "failures" => super::project::failures(places),
        "start" => start::start(places),
        "sessions" => sessions::sessions(places),
        running => match herdr_view(&places.herdr, asked.state.is_some()) {
            None => vec![notice(
                "herdr isn't running",
                "Agents, tabs and recent places come from herdr. Instructions, failures, start and sessions work anywhere.",
            )],
            Some(Err(failure)) => vec![notice("herdr did not answer", &failure.to_string())],
            Some(Ok(view)) => running::rows(running, &view, places.home.as_deref()),
        },
    };
    let refresh = (key("ctrl-r"), Action::State(FRESH.to_string()));
    listed
        .into_iter()
        .map(|row| PickerRow {
            keys: row.keys.iter().cloned().chain([refresh.clone()]).collect(),
            ..row
        })
        .collect()
}

#[cfg(test)]
mod tests;
