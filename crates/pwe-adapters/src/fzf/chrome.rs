use pwe_domain::{PickerRow, Theme, View, ViewResponse, shown};

use super::Helper;
use crate::screen::palette;

const DIM: &str = "\x1b[2m";
const UNDIM: &str = "\x1b[22m";
const RESET: &str = "\x1b[0m";

pub fn list_lines(rows: &[PickerRow], view: &View) -> Vec<String> {
    let shown = shown(rows, view);
    let column = shown
        .iter()
        .filter_map(|(_, row)| row.group.as_ref())
        .map(|group| group.chars().count())
        .max();
    shown
        .into_iter()
        .map(|(index, row)| match column {
            Some(width) => format!(
                "{index}\t{DIM}{:<width$}{UNDIM}  {}",
                row.group.as_deref().unwrap_or_default(),
                row.label
            ),
            None => format!("{index}\t{}", row.label),
        })
        .collect()
}

pub const SWITCH_SOURCE: &str = "source";

pub const SAVE_QUICKFIX: &str = "quickfix";

pub const SEND_ROWS: &str = "send";

pub fn chooser_lines(entries: &[&str]) -> Vec<String> {
    entries
        .iter()
        .enumerate()
        .map(|(index, entry)| format!("{index}\t{entry}"))
        .collect()
}

pub fn finder_actions(response: ViewResponse, helper: Helper, failed: bool, live: bool) -> String {
    let reload = format!("reload({})", helper.call("rows", ""));
    let mut actions: Vec<String> = match response {
        ViewResponse::Redraw => vec![reload],
        ViewResponse::OpenChooser => vec![reload, "clear-query".into(), "hide-preview".into()],
        ViewResponse::CloseChooser => vec![reload, "clear-query".into(), "show-preview".into()],
        ViewResponse::Accept => vec!["accept".into()],
        ViewResponse::Back => vec![leave("esc")],
        ViewResponse::Switch(_) => vec![leave(SWITCH_SOURCE)],
        ViewResponse::Stay => Vec::new(),
    };
    match (&response, live) {
        (ViewResponse::OpenChooser, true) => actions.push("enable-search".into()),
        (ViewResponse::CloseChooser, true) => actions.push("disable-search".into()),
        _ => {}
    }
    if failed {
        actions.push(format!("transform-footer({})", helper.call("footer", "")));
    }
    actions.join("+")
}

pub fn leave(key: &str) -> String {
    format!("print({key})+accept")
}

pub fn change_footer(text: &str) -> String {
    format!("change-footer:{text}")
}

pub fn step_footer(
    hint: &[String],
    description: &str,
    error: &str,
    width: usize,
    theme: Theme,
) -> String {
    let error = (!error.is_empty()).then(|| error_footer(error, width, theme));
    packed(hint.iter().map(String::as_str), " · ", width)
        .into_iter()
        .chain((!description.is_empty()).then(|| description.to_string()))
        .chain(error)
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn help_page(header: &str, entries: &[(String, String)], theme: Theme) -> String {
    let palette = palette(theme);
    let width = entries
        .iter()
        .map(|(key, _)| key.chars().count())
        .max()
        .unwrap_or_default();
    let lines = entries
        .iter()
        .map(|(key, text)| format!("{}{key:<width$}{RESET}  {text}", palette.key));
    std::iter::once(format!("{}{header} › Help{RESET}", palette.faint))
        .chain(std::iter::once(String::new()))
        .chain(lines)
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn error_footer(error: &str, width: usize, theme: Theme) -> String {
    let red = palette(theme).refusal;
    packed(error.split_whitespace(), " ", width)
        .iter()
        .map(|line| format!("{red}{line}{RESET}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn packed<'a>(pieces: impl Iterator<Item = &'a str>, separator: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for piece in pieces {
        match lines.last_mut() {
            Some(line)
                if line.chars().count() + separator.chars().count() + piece.chars().count()
                    <= width =>
            {
                line.push_str(separator);
                line.push_str(piece);
            }
            _ => lines.push(piece.to_string()),
        }
    }
    lines
}

#[cfg(test)]
mod tests;
