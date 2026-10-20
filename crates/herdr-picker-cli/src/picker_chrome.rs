use std::env;
use std::path::Path;

use herdr_picker_adapters::{
    Helper, cached_rows, chooser_lines, error_footer, finder_actions, list_lines, read_config,
    read_run_error, read_view, write_view,
};
use herdr_picker_domain::{PickerRow, Theme, View, ViewKey, groups, header, press};
use herdr_picker_protocol::shell_quote;

use crate::command::{CommandError, failed};

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
    match (part, rest) {
        ("header", [name]) => picker_header(cache, name),
        ("rows", []) => view_rows(cache),
        ("view", event) => view_key(cache, event),
        ("footer", []) => Ok(run_error_footer(
            cache,
            env::var("FZF_COLUMNS")
                .ok()
                .and_then(|columns| columns.parse().ok()),
            read_config(config)
                .map(|config| config.theme)
                .unwrap_or_default(),
        )),
        _ => Err(CommandError::Usage),
    }
}

fn picker_header(cache: &Path, name: &str) -> Result<String, CommandError> {
    let rows = rows_of(cache)?;
    Ok(header(name, &read_view(cache), &groups(&rows)))
}

fn view_rows(cache: &Path) -> Result<String, CommandError> {
    let rows = rows_of(cache)?;
    let lines = match read_view(cache) {
        View::Chooser { .. } => chooser_lines(&groups(&rows)),
        view => list_lines(&rows, &view),
    };
    Ok(lines.into_iter().map(|line| line + "\n").collect())
}

fn view_key(cache: &Path, event: &[&str]) -> Result<String, CommandError> {
    let key = match event {
        ["load"] => ViewKey::Loaded,
        ["choose"] => ViewKey::Choose,
        ["enter", field] => ViewKey::Enter(field.trim().parse().ok()),
        ["back"] => ViewKey::Back,
        _ => return Err(CommandError::Usage),
    };
    let (view, response) = press(&read_view(cache), key, &groups(&rows_of(cache)?));
    write_view(cache, &view).map_err(failed)?;
    let failed_run = key == ViewKey::Loaded && read_run_error(cache).is_some();
    let quoted = QuotedHelper::for_run(cache)?;
    Ok(finder_actions(response, quoted.helper(), failed_run))
}

// fzf hands over its whole width; with the preview showing, the footer gets about half.
fn run_error_footer(cache: &Path, columns: Option<usize>, theme: Theme) -> String {
    let width = columns.map_or(FOOTER_WIDTH, |columns| (columns / 2).saturating_sub(6));
    error_footer(
        &read_run_error(cache).unwrap_or_default(),
        width.max(FOOTER_WIDTH),
        theme,
    )
}

const FOOTER_WIDTH: usize = 20;

fn rows_of(cache: &Path) -> Result<Vec<PickerRow>, CommandError> {
    cached_rows(cache).map_err(failed)
}

#[cfg(test)]
mod tests;
