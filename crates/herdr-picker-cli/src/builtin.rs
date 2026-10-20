use herdr_picker_domain::{Action, PickerRow};
use herdr_picker_protocol::{encode_row, parse_action, shell_word};

use crate::command::{CommandError, failed};

// herdr-picker's own pickers print their rows like any picker command.
pub(crate) fn rows(name: &str, arguments: &[&str]) -> Result<String, CommandError> {
    let rows = match (name, arguments) {
        ("echo-args", words) => words
            .iter()
            .map(|word| row(word, None, Action::Insert(shell_word(word))))
            .collect(),
        ("confirm", [question, action]) => {
            let action = parse_action(action).map_err(failed)?;
            vec![
                row("Yes", Some(question), action),
                row("No", Some(question), Action::Back),
            ]
        }
        _ => return Err(failed(format!("no builtin picker is named {name}"))),
    };
    Ok(rows.iter().map(|row| encode_row(row) + "\n").collect())
}

fn row(label: &str, preview: Option<&str>, enter: Action) -> PickerRow {
    PickerRow {
        label: label.to_string(),
        preview: preview.map(str::to_string),
        enter,
        keys: Vec::new(),
        group: None,
    }
}

#[cfg(test)]
mod tests;
