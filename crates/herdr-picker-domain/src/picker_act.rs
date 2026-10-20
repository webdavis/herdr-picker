use std::fmt;
use std::mem::discriminant;

use crate::picker_key::Key;
use crate::picker_row::{Action, PickerCall, PickerRow};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActRefusal {
    SelectOne { purpose: &'static str },
    Mixed { key: String },
}

impl fmt::Display for ActRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SelectOne { purpose } => write!(formatter, "Select one row to {purpose}"),
            Self::Mixed { key } => {
                write!(formatter, "The marked rows do different things on {key}")
            }
        }
    }
}

// What a key does to the marked rows, or to the highlighted one when none is marked.
pub fn act(rows: &[&PickerRow], key: &Key) -> Result<Action, ActRefusal> {
    let actions: Vec<&Action> = rows.iter().filter_map(|row| row.action(key)).collect();
    let Some(first) = actions.first().copied() else {
        return Ok(Action::Nothing);
    };
    let alike = actions
        .iter()
        .all(|action| discriminant(*action) == discriminant(first));
    if actions.len() < rows.len() || !alike {
        return Err(ActRefusal::Mixed {
            key: key.label().to_string(),
        });
    }
    if actions.len() == 1 {
        return Ok(first.clone());
    }
    let joined = |separator: &str| {
        let texts: Vec<&str> = actions.iter().filter_map(|action| text(action)).collect();
        texts.join(separator)
    };
    let select_one = |purpose| Err(ActRefusal::SelectOne { purpose });
    match first {
        Action::Insert(_) => Ok(Action::Insert(joined(" "))),
        Action::Copy(_) => Ok(Action::Copy(joined("\n"))),
        Action::CopyStay(_) => Ok(Action::CopyStay(joined("\n"))),
        Action::Edit(_) => Ok(Action::Edit(
            actions
                .iter()
                .flat_map(|action| match action {
                    Action::Edit(locations) => locations.clone(),
                    _ => Vec::new(),
                })
                .collect(),
        )),
        Action::Next(first) => merged_next(first, &actions).ok_or(ActRefusal::SelectOne {
            purpose: "go to its picker",
        }),
        Action::Source(_) | Action::State(_) | Action::Reload | Action::Back | Action::Nothing => {
            Ok(first.clone())
        }
        Action::Cd(_) => select_one("change into"),
        Action::Run(_) => select_one("run"),
        Action::Open(_) => select_one("open"),
        Action::Confirm { .. } => select_one("confirm"),
    }
}

// One step for every marked row, so it can act on all of them: their args in the order marked.
fn merged_next(first: &PickerCall, actions: &[&Action]) -> Option<Action> {
    let mut args = Vec::new();
    for action in actions {
        let Action::Next(call) = action else {
            return None;
        };
        let alike =
            (&call.name, &call.state, &call.query) == (&first.name, &first.state, &first.query);
        if !alike {
            return None;
        }
        args.extend(call.args.iter().cloned());
    }
    Some(Action::Next(PickerCall {
        args,
        ..first.clone()
    }))
}

fn text(action: &Action) -> Option<&str> {
    match action {
        Action::Insert(text) | Action::Copy(text) | Action::CopyStay(text) => Some(text),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
