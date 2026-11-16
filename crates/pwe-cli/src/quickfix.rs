use std::env;
use std::path::{Path, PathBuf};

use pwe_adapters::{
    SAVE_QUICKFIX, change_footer, error_footer, leave, quickfix_file, save_quickfix, send_quickfix,
};
use pwe_domain::{PickerRow, Theme, quickfix_entries};

use crate::command::{CommandError, state_dir};
use crate::picker_chrome::rows_of;
use crate::picker_keys::indices;

pub(crate) struct Quickfix {
    pub(crate) file: PathBuf,
    pub(crate) nvim: Option<String>,
}

impl Quickfix {
    pub(crate) fn here() -> Self {
        Self {
            file: quickfix_path(),
            nvim: env::var("NVIM").ok().filter(|socket| !socket.is_empty()),
        }
    }
}

pub(crate) fn quickfix_path() -> PathBuf {
    quickfix_file(&state_dir())
}

pub(crate) fn save_marked(
    cache: &Path,
    fields: &[&str],
    quickfix: &Quickfix,
    theme: Theme,
) -> Result<String, CommandError> {
    let rows = rows_of(cache)?;
    let marked: Vec<&PickerRow> = indices(fields)
        .iter()
        .filter_map(|index| rows.get(*index))
        .collect();
    let refused = |reason: &str| change_footer(&error_footer(reason, usize::MAX, theme));
    let entries = quickfix_entries(&marked);
    if entries.is_empty() {
        return Ok(refused("No file locations in the selection."));
    }
    if let Err(failure) = save_quickfix(&quickfix.file, &entries) {
        return Ok(refused(&format!(
            "Cannot save the quickfix list: {failure}"
        )));
    }
    let Some(socket) = &quickfix.nvim else {
        return Ok(leave(SAVE_QUICKFIX));
    };
    Ok(match send_quickfix(socket, &quickfix.file) {
        Ok(()) => change_footer(&match entries.len() {
            1 => "Saved 1 location to the quickfix list".to_string(),
            count => format!("Saved {count} locations to the quickfix list"),
        }),
        Err(failure) => refused(&format!("Cannot reach Neovim: {failure}")),
    })
}

#[cfg(test)]
mod tests;
