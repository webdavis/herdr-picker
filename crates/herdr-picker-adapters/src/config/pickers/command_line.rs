use std::path::Path;

use herdr_picker_domain::PickerError;
use herdr_picker_protocol::split_words;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(
    untagged,
    expecting = "a command line, or an array of a program and its arguments"
)]
pub(super) enum CommandLine {
    Line(String),
    Words(Vec<String>),
}

// A relative program is one of the scripts in the pickers directory; `name` names the refusal.
pub(super) fn program_words(
    line: CommandLine,
    pickers_directory: &Path,
    name: &str,
) -> Result<Vec<String>, PickerError> {
    let refusal = || PickerError::UnusableCommand {
        name: name.to_string(),
        command: match &line {
            CommandLine::Line(text) => text.clone(),
            CommandLine::Words(words) => words.join(" "),
        },
    };
    let mut words = match &line {
        CommandLine::Line(text) => split_words(text).map_err(|_| refusal())?,
        CommandLine::Words(words) => words.clone(),
    };
    let first = words
        .first_mut()
        .filter(|first| !first.is_empty())
        .ok_or_else(refusal)?;
    *first = pickers_directory
        .join(&*first)
        .to_string_lossy()
        .into_owned();
    Ok(words)
}
