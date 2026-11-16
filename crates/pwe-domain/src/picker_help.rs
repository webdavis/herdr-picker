use crate::new_pane::NewPane;
use crate::picker_key::Key;
use crate::picker_quickfix::quickfix_entries;
use crate::picker_row::PickerRow;

mod verbs;

use verbs::{describe, detail, verb};

const HELP_KEY: &str = "F1 help";
const SEND_KEY: &str = "alt-o send";
const QUICKFIX_KEY: &str = "ctrl-q quickfix";
const CHOOSE: (&str, &str) = ("ctrl-space", "choose a source");
const GENERAL: [(&str, &str); 5] = [
    ("tab", "mark a row; a key then acts on every marked row"),
    (
        "ctrl-q",
        "save the marked rows' files and lines as Neovim's quickfix list and open it",
    ),
    (
        "alt-o",
        "send the marked rows to an agent, copy them, or ask a quick question about them",
    ),
    ("esc", "back to the previous step, or close on the first"),
    ("F1", "this help"),
];

// One short entry a key, for the footer to lay out.
pub fn footer_hint(row: &PickerRow) -> Vec<String> {
    let claimed = row.offered().filter_map(|(key, action)| {
        let does = key.label().or_else(|| verb(action))?;
        Some(format!("{} {does}", key.shown()))
    });
    let (panes, tab): (Vec<_>, Vec<_>) =
        new_panes(row).partition(|(_, pane)| *pane != NewPane::Tab);
    let letters: Vec<&str> = panes
        .iter()
        .filter_map(|(key, _)| key.name().strip_prefix("alt-"))
        .collect();
    let panes = (!letters.is_empty()).then(|| format!("alt-{} new pane", letters.join("/")));
    let tab = tab
        .first()
        .map(|(key, _)| format!("{} new tab", key.shown()));
    let quickfix = (!quickfix_entries(&[row]).is_empty()).then(|| QUICKFIX_KEY.to_string());
    claimed
        .chain(panes)
        .chain(tab)
        .chain(quickfix)
        .chain([SEND_KEY, HELP_KEY].map(str::to_string))
        .collect()
}

// Each key the row claims, the new-pane keys it leaves free, then the keys every picker has.
pub fn help_entries(row: Option<&PickerRow>, grouped: bool) -> Vec<(String, String)> {
    let claimed = row.into_iter().flat_map(|row| {
        row.offered().filter_map(|(key, action)| {
            let text = match (key.label(), detail(action)) {
                (Some(label), Some(detail)) => format!("{label} {detail}"),
                (Some(label), None) => label.to_string(),
                (None, _) => describe(action)?,
            };
            Some((key.shown().to_string(), text))
        })
    });
    let panes = row.into_iter().flat_map(|row| {
        new_panes(row).map(|(key, pane)| (key.shown().to_string(), opens_in(pane).to_string()))
    });
    let general = grouped
        .then_some(CHOOSE)
        .into_iter()
        .chain(GENERAL)
        .map(|(key, text)| (key.to_string(), text.to_string()));
    claimed.chain(panes).chain(general).collect()
}

fn new_panes(row: &PickerRow) -> impl Iterator<Item = (Key, NewPane)> + '_ {
    NewPane::keys().filter(|(key, _)| NewPane::takes(&row.enter) && row.action(key).is_none())
}

fn opens_in(pane: NewPane) -> &'static str {
    match pane {
        NewPane::Left => "open the result in a new pane to the left",
        NewPane::Below => "open the result in a new pane below",
        NewPane::Above => "open the result in a new pane above",
        NewPane::Right => "open the result in a new pane to the right",
        NewPane::Tab => "open the result in a new tab",
    }
}

#[cfg(test)]
mod tests;
