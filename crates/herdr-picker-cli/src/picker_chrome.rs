use std::env;
use std::path::Path;

use herdr_picker_adapters::{
    Helper, cached_rows, chooser_lines, help_page, list_lines, read_config, read_run_error,
    read_view, step_footer,
};
use herdr_picker_domain::{PickerRow, Theme, View, footer_hint, groups, header, help_entries};
use herdr_picker_protocol::shell_quote;

use crate::command::{CommandError, failed};
use crate::picker_keys;

pub(crate) struct QuotedHelper {
    program: String,
    cache: String,
}

impl QuotedHelper {
    pub(crate) fn for_run(cache: &Path) -> Result<Self, CommandError> {
        Ok(Self {
            program: shell_quote(&env::current_exe().map_err(failed)?.to_string_lossy()),
            cache: shell_quote(&cache.to_string_lossy()),
        })
    }

    pub(crate) fn helper(&self) -> Helper<'_> {
        Helper {
            program: &self.program,
            cache: &self.cache,
        }
    }
}

pub(crate) fn answer(
    part: &str,
    cache: &Path,
    rest: &[&str],
    config: &Path,
) -> Result<String, CommandError> {
    let theme = read_config(config)
        .map(|config| config.theme)
        .unwrap_or_default();
    match (part, rest) {
        ("header", []) => picker_header(cache),
        ("rows", []) => view_rows(cache),
        ("view", event) => picker_keys::view_key(cache, event, theme),
        ("footer", [] | [_]) => picker_footer(cache, rest.first().copied(), columns(), theme),
        ("help", [] | [_]) => picker_help(cache, rest.first().copied(), theme),
        _ => Err(CommandError::Usage),
    }
}

fn picker_header(cache: &Path) -> Result<String, CommandError> {
    let state = read_view(cache);
    Ok(header(
        &state.titles,
        &state.view,
        &groups(&rows_of(cache)?),
    ))
}

fn view_rows(cache: &Path) -> Result<String, CommandError> {
    let rows = rows_of(cache)?;
    let lines = match read_view(cache).view {
        View::Chooser { .. } => chooser_lines(&groups(&rows)),
        view => list_lines(&rows, &view),
    };
    Ok(lines.into_iter().map(|line| line + "\n").collect())
}

// fzf hands over its whole width; with the preview showing, the footer gets about half.
fn picker_footer(
    cache: &Path,
    index: Option<&str>,
    columns: Option<usize>,
    theme: Theme,
) -> Result<String, CommandError> {
    let width = columns.map_or(FOOTER_WIDTH, |columns| (columns / 2).saturating_sub(6));
    let hint = row_at(cache, index)?.map(|row| footer_hint(&row));
    Ok(step_footer(
        hint.as_deref().unwrap_or_default(),
        &read_view(cache).description,
        &read_run_error(cache).unwrap_or_default(),
        width.max(FOOTER_WIDTH),
        theme,
    ))
}

fn picker_help(cache: &Path, index: Option<&str>, theme: Theme) -> Result<String, CommandError> {
    let grouped = !groups(&rows_of(cache)?).is_empty();
    let entries = help_entries(row_at(cache, index)?.as_ref(), grouped);
    Ok(help_page(&picker_header(cache)?, &entries, theme))
}

// The highlighted row, which the chooser's entries are not.
fn row_at(cache: &Path, index: Option<&str>) -> Result<Option<PickerRow>, CommandError> {
    if matches!(read_view(cache).view, View::Chooser { .. }) {
        return Ok(None);
    }
    let index = index.and_then(|index| index.trim().parse::<usize>().ok());
    Ok(index.and_then(|index| rows_of(cache).ok()?.into_iter().nth(index)))
}

fn columns() -> Option<usize> {
    env::var("FZF_COLUMNS")
        .ok()
        .and_then(|columns| columns.parse().ok())
}

const FOOTER_WIDTH: usize = 20;

pub(crate) fn rows_of(cache: &Path) -> Result<Vec<PickerRow>, CommandError> {
    cached_rows(cache).map_err(failed)
}

#[cfg(test)]
mod tests;
