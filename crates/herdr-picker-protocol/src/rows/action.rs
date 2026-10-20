use herdr_picker_domain::{Action, Location, PickerCall};

use super::wire::{
    Bare, OneOrMany, Tagged, WireAction, WireConfirm, WireLocation, WireNext, Words,
};
use crate::shell::shell_word;

// A list of words is inserted as shell words; one text is inserted as it is.
fn inserted(words: Words) -> String {
    match words {
        Words::One(text) => text,
        Words::Many(words) => words
            .iter()
            .map(|word| shell_word(word))
            .collect::<Vec<_>>()
            .join(" "),
    }
}

fn copied(words: Words) -> String {
    match words {
        Words::One(text) => text,
        Words::Many(lines) => lines.join("\n"),
    }
}

pub(super) fn into_action(wire: WireAction, value: Option<&Words>) -> Result<Action, String> {
    let value = || {
        value
            .cloned()
            .ok_or_else(|| "an action uses the row's value, but the row has none".to_string())
    };
    Ok(match wire {
        WireAction::Bare(Bare::Insert) => Action::Insert(inserted(value()?)),
        WireAction::Bare(Bare::Copy) => Action::Copy(copied(value()?)),
        WireAction::Bare(Bare::CopyStay) => Action::CopyStay(copied(value()?)),
        WireAction::Bare(Bare::Reload) => Action::Reload,
        WireAction::Bare(Bare::None) => Action::Nothing,
        WireAction::Bare(Bare::Back) => Action::Back,
        WireAction::Tagged(tagged) => match tagged {
            Tagged::Insert(words) => Action::Insert(inserted(words)),
            Tagged::Copy(words) => Action::Copy(copied(words)),
            Tagged::CopyStay(words) => Action::CopyStay(copied(words)),
            Tagged::Cd(path) => Action::Cd(path),
            Tagged::Run(command) => Action::Run(command),
            Tagged::Open(target) => Action::Open(target),
            Tagged::Edit(OneOrMany::One(location)) => Action::Edit(vec![location.into()]),
            Tagged::Edit(OneOrMany::Many(locations)) => {
                Action::Edit(locations.into_iter().map(Location::from).collect())
            }
            Tagged::Next(next) => Action::Next(PickerCall {
                name: next.picker,
                args: next.args,
                state: next.state.map(|state| state.to_string()),
                query: next.query,
            }),
            Tagged::Confirm(confirm) => Action::Confirm {
                question: confirm.question,
                action: Box::new(into_action(*confirm.action, None)?),
            },
            Tagged::Source(source) => Action::Source(source),
            Tagged::State(state) => Action::State(state.to_string()),
        },
    })
}

impl From<WireLocation> for Location {
    fn from(wire: WireLocation) -> Self {
        Self {
            path: wire.path,
            line: wire.line,
        }
    }
}

pub(super) fn from_action(action: &Action) -> WireAction {
    let one = |text: &String| Words::One(text.clone());
    WireAction::Tagged(match action {
        Action::Insert(text) => Tagged::Insert(one(text)),
        Action::Copy(text) => Tagged::Copy(one(text)),
        Action::CopyStay(text) => Tagged::CopyStay(one(text)),
        Action::Cd(path) => Tagged::Cd(path.clone()),
        Action::Run(command) => Tagged::Run(command.clone()),
        Action::Open(target) => Tagged::Open(target.clone()),
        Action::Edit(locations) => Tagged::Edit(OneOrMany::Many(
            locations
                .iter()
                .map(|location| WireLocation {
                    path: location.path.clone(),
                    line: location.line,
                })
                .collect(),
        )),
        Action::Next(call) => Tagged::Next(WireNext {
            picker: call.name.clone(),
            args: call.args.clone(),
            state: call
                .state
                .as_deref()
                .and_then(|state| serde_json::from_str(state).ok()),
            query: call.query.clone(),
        }),
        Action::Confirm { question, action } => Tagged::Confirm(WireConfirm {
            question: question.clone(),
            action: Box::new(from_action(action)),
        }),
        Action::Source(source) => Tagged::Source(source.clone()),
        Action::State(state) => {
            Tagged::State(serde_json::from_str(state).unwrap_or(serde_json::Value::Null))
        }
        Action::Reload => return WireAction::Bare(Bare::Reload),
        Action::Nothing => return WireAction::Bare(Bare::None),
        Action::Back => return WireAction::Bare(Bare::Back),
        Action::InPane { action, .. } => return from_action(action),
    })
}
