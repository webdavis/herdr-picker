use pwe_domain::{PickerRow, View, ViewResponse};

use super::Helper;
use super::ending::{BACK, SWITCH_SOURCE};
use crate::picker_text::listed_labels;

pub fn list_lines(rows: &[PickerRow], view: &View) -> Vec<String> {
    listed_labels(rows, view)
        .into_iter()
        .map(|(index, label)| format!("{index}\t{label}"))
        .collect()
}

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
        ViewResponse::Back => vec![leave(BACK)],
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

#[cfg(test)]
mod tests;
