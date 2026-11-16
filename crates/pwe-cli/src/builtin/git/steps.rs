use std::io::Write;

use pwe_domain::{Action, PickerCall, PickerRow, Preview};

use super::paint::dim;
use crate::builtin::finder_closing_is_no_failure;
use crate::builtin::rows::RowWriter;
use crate::command::CommandError;

pub(super) fn next(
    name: &str,
    args: &[&str],
    query: Option<&str>,
    state: Option<String>,
) -> Action {
    Action::Next(PickerCall {
        name: name.to_string(),
        args: args.iter().map(|arg| arg.to_string()).collect(),
        state,
        query: query.map(str::to_string),
    })
}

pub(super) fn confirm(question: &str, command: String) -> Action {
    Action::Confirm {
        question: question.to_string(),
        action: Box::new(Action::Run(command)),
    }
}

pub(super) struct Choice {
    pub(super) label: String,
    pub(super) note: String,
    pub(super) action: Action,
}

pub(super) fn choice(label: &str, note: &str, action: Action) -> Choice {
    Choice {
        label: label.to_string(),
        note: note.to_string(),
        action,
    }
}

pub(super) fn ask(
    question: &str,
    detail: &str,
    choices: Vec<Choice>,
    rows: &mut RowWriter<impl Write>,
) -> Result<(), CommandError> {
    finder_closing_is_no_failure(rows.header(question))?;
    let cancel = choice("Cancel", "nothing changes", Action::Back);
    finder_closing_is_no_failure(
        std::iter::once(cancel)
            .chain(choices)
            .try_for_each(|choice| {
                rows.row(&PickerRow {
                    label: match choice.note.is_empty() {
                        true => choice.label,
                        false => format!("{}  {}", choice.label, dim(&choice.note)),
                    },
                    preview: Some(Preview::Text(detail.to_string())),
                    enter: choice.action,
                    keys: Vec::new(),
                    group: None,
                })
            }),
    )
}
