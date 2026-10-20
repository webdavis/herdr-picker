use herdr_picker_domain::{PickerRow, Theme, View, ViewResponse, chooser_entries, shown};

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

pub fn chooser_lines(groups: &[String]) -> Vec<String> {
    chooser_entries(groups)
        .into_iter()
        .enumerate()
        .map(|(index, entry)| format!("{index}\t{entry}"))
        .collect()
}

pub fn finder_actions(response: ViewResponse, helper: Helper, failed: bool) -> String {
    let reload = format!("reload({})", helper.call("rows", ""));
    let mut actions: Vec<String> = match response {
        ViewResponse::Redraw => vec![reload],
        ViewResponse::OpenChooser => vec![reload, "clear-query".into(), "hide-preview".into()],
        ViewResponse::CloseChooser => vec![reload, "clear-query".into(), "show-preview".into()],
        ViewResponse::Accept => vec!["accept".into()],
        ViewResponse::Abort => vec!["abort".into()],
        ViewResponse::Stay => Vec::new(),
    };
    if failed {
        actions.push(format!("transform-footer({})", helper.call("footer", "")));
    }
    actions.join("+")
}

pub fn error_footer(error: &str, width: usize, theme: Theme) -> String {
    let red = palette(theme).refusal;
    let mut lines: Vec<String> = Vec::new();
    for word in error.split_whitespace() {
        match lines.last_mut() {
            Some(line) if line.chars().count() + 1 + word.chars().count() <= width => {
                line.push(' ');
                line.push_str(word);
            }
            _ => lines.push(word.to_string()),
        }
    }
    lines
        .iter()
        .map(|line| format!("{red}{line}{RESET}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests;
