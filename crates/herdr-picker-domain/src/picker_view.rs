use crate::picker_row::PickerRow;

const EVERY_GROUP: &str = "all";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View {
    // The rows as the command printed them, before the list draws a group column.
    Streamed,
    List { group: Option<String> },
    Chooser { group: Option<String> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewKey {
    Loaded,
    Choose,
    Enter(Option<usize>),
    Back,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewResponse {
    Redraw,
    OpenChooser,
    CloseChooser,
    Accept,
    Abort,
    Stay,
}

pub fn groups(rows: &[PickerRow]) -> Vec<String> {
    let mut groups: Vec<String> = rows.iter().filter_map(|row| row.group.clone()).collect();
    groups.sort();
    groups.dedup();
    groups
}

pub fn chooser_entries(groups: &[String]) -> Vec<&str> {
    std::iter::once(EVERY_GROUP)
        .chain(groups.iter().map(String::as_str))
        .collect()
}

pub fn shown<'a>(rows: &'a [PickerRow], view: &View) -> Vec<(usize, &'a PickerRow)> {
    let only = match view {
        View::List { group: Some(group) } => Some(group),
        _ => None,
    };
    rows.iter()
        .enumerate()
        .filter(|(_, row)| only.is_none_or(|group| row.group.as_ref() == Some(group)))
        .collect()
}

pub fn header(name: &str, view: &View, groups: &[String]) -> String {
    let title = title(name);
    match view {
        View::Chooser { .. } => format!("{title} › Choose source"),
        _ if groups.is_empty() => title,
        View::List { group: Some(group) } => format!("{title} | Source: {group}"),
        _ => format!("{title} | Source: {EVERY_GROUP}"),
    }
}

fn title(name: &str) -> String {
    name.split(['-', '_'])
        .map(|word| {
            let mut letters = word.chars();
            letters
                .next()
                .map(|first| first.to_uppercase().chain(letters).collect())
                .unwrap_or_default()
        })
        .collect::<Vec<String>>()
        .join(" ")
}

pub fn press(view: &View, key: ViewKey, groups: &[String]) -> (View, ViewResponse) {
    let stay = (view.clone(), ViewResponse::Stay);
    match (view, key) {
        (View::Streamed, ViewKey::Loaded) if !groups.is_empty() => {
            (View::List { group: None }, ViewResponse::Redraw)
        }
        (View::List { group }, ViewKey::Choose) => (
            View::Chooser {
                group: group.clone(),
            },
            ViewResponse::OpenChooser,
        ),
        (View::Chooser { .. }, ViewKey::Enter(Some(0))) => {
            (View::List { group: None }, ViewResponse::CloseChooser)
        }
        (View::Chooser { .. }, ViewKey::Enter(Some(entry))) => match groups.get(entry - 1) {
            Some(group) => (
                View::List {
                    group: Some(group.clone()),
                },
                ViewResponse::CloseChooser,
            ),
            None => stay,
        },
        (View::Chooser { group }, ViewKey::Back) => (
            View::List {
                group: group.clone(),
            },
            ViewResponse::CloseChooser,
        ),
        (View::Streamed | View::List { .. }, ViewKey::Enter(_)) => {
            (view.clone(), ViewResponse::Accept)
        }
        (View::Streamed | View::List { .. }, ViewKey::Back) => (view.clone(), ViewResponse::Abort),
        _ => stay,
    }
}

#[cfg(test)]
mod tests;
