use std::process::Command;

use herdr_picker_domain::{Action, PickerRow, Preview};

use super::sending;
use crate::builtin::Asked;
use crate::builtin::agents::start::{BROCCOLI, PERSONAL};
use crate::builtin::page::notice;

pub(in crate::builtin) fn answer(asked: &Asked) -> Vec<PickerRow> {
    let sending = sending(asked);
    answer_page(&quick_answer_argv(
        &sending.account,
        &sending.question,
        &sending.text,
    ))
}

pub(in crate::builtin) fn quick_answer_argv(
    account: &str,
    question: &str,
    text: &str,
) -> Vec<String> {
    let profile = if account == BROCCOLI {
        BROCCOLI
    } else {
        PERSONAL
    };
    let prompt = format!("{question}\n\nThe rows I picked:\n{text}");
    ["clauth", "start", profile, "-p", &prompt]
        .map(str::to_string)
        .to_vec()
}

// The answer a line a row; Enter copies all of it and Esc goes back to the question.
pub(in crate::builtin) fn answer_page(argv: &[String]) -> Vec<PickerRow> {
    let Some((program, arguments)) = argv.split_first() else {
        return Vec::new();
    };
    let ran = match Command::new(program).args(arguments).output() {
        Ok(ran) => ran,
        Err(failure) => {
            return vec![notice(
                &format!("Cannot run {program}"),
                &failure.to_string(),
            )];
        }
    };
    let answer = String::from_utf8_lossy(&ran.stdout).trim_end().to_string();
    if !ran.status.success() || answer.is_empty() {
        let said = String::from_utf8_lossy(&ran.stderr).into_owned();
        return vec![notice("No answer came back", &said)];
    }
    answer
        .lines()
        .map(|line| PickerRow {
            label: line.to_string(),
            preview: Some(Preview::Text(answer.clone())),
            enter: Action::Copy(answer.clone()),
            keys: Vec::new(),
            group: None,
        })
        .collect()
}
