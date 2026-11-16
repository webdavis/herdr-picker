use std::path::Path;

use pwe_adapters::{inbox_file, inbox_listing_json, read_inbox};
use pwe_domain::newest_first;

use crate::command::{CommandError, state_dir};

const UNREAD_ONLY: &str = "--unread";

pub(crate) fn run(options: &[&str]) -> Result<(), CommandError> {
    println!("{}", listing(&inbox_file(&state_dir()), options)?);
    Ok(())
}

pub(crate) fn listing(file: &Path, options: &[&str]) -> Result<String, CommandError> {
    let unread_only = match options {
        [] => false,
        [UNREAD_ONLY] => true,
        _ => return Err(CommandError::Usage),
    };
    let rows = read_inbox(file).map_err(|failure| {
        CommandError::Failed(format!(
            "could not read the inbox {}: {failure}",
            file.display()
        ))
    })?;
    let listed: Vec<_> = newest_first(rows)
        .into_iter()
        .filter(|row| !unread_only || row.is_unread())
        .collect();
    inbox_listing_json(&listed)
        .map_err(|failure| CommandError::Failed(format!("could not print the inbox: {failure}")))
}

#[cfg(test)]
mod tests;
