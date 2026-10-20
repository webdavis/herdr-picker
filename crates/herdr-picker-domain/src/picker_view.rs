use crate::picker_row::PickerRow;

pub(crate) const EVERY_GROUP: &str = "all";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View {
    // The rows as the command printed them, before the list draws a group column; the group is
    // the one to filter to once they load, when Esc came back to this step.
    Streamed { group: Option<String> },
    List { group: Option<String> },
    Chooser { group: Option<String> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewKey {
    Loaded,
    Choose,
    Enter(Option<usize>),
    // A row's key other than enter.
    Other,
    Back,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewResponse {
    Redraw,
    OpenChooser,
    CloseChooser,
    // Leave this run so the step shows again, its rows read from the named source.
    Switch(String),
    Accept,
    Back,
    Stay,
}

// What the callbacks fzf runs during one step need to know about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewState {
    pub view: View,
    pub titles: Vec<String>,
    pub description: String,
    pub source: Option<String>,
    pub sources: Vec<String>,
}

pub fn groups(rows: &[PickerRow]) -> Vec<String> {
    let mut groups: Vec<String> = rows.iter().filter_map(|row| row.group.clone()).collect();
    groups.sort();
    groups.dedup();
    groups
}

// The sources first, then every group when the rows carry them.
pub fn chooser_entries<'a>(sources: &'a [String], groups: &'a [String]) -> Vec<&'a str> {
    let grouped = (!groups.is_empty()).then_some(EVERY_GROUP);
    sources
        .iter()
        .map(String::as_str)
        .chain(grouped)
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

pub fn press(
    view: &View,
    key: ViewKey,
    sources: &[String],
    groups: &[String],
) -> (View, ViewResponse) {
    let stay = (view.clone(), ViewResponse::Stay);
    let open = |group: &Option<String>| {
        let group = group.clone();
        (View::Chooser { group }, ViewResponse::OpenChooser)
    };
    match (view, key) {
        (View::Streamed { group }, ViewKey::Loaded) if !groups.is_empty() => (
            View::List {
                group: group.clone().filter(|group| groups.contains(group)),
            },
            ViewResponse::Redraw,
        ),
        (View::List { group }, ViewKey::Choose) => open(group),
        // Sources are known before the rows load, so they can be switched while rows still stream.
        (View::Streamed { group }, ViewKey::Choose) if !sources.is_empty() => open(group),
        (View::Chooser { .. }, ViewKey::Enter(Some(entry))) => {
            chosen(entry, sources, groups).unwrap_or(stay)
        }
        (View::Chooser { group }, ViewKey::Back) => (
            View::List {
                group: group.clone(),
            },
            ViewResponse::CloseChooser,
        ),
        (View::Streamed { .. } | View::List { .. }, ViewKey::Enter(_) | ViewKey::Other) => {
            (view.clone(), ViewResponse::Accept)
        }
        (View::Streamed { .. } | View::List { .. }, ViewKey::Back) => {
            (view.clone(), ViewResponse::Back)
        }
        _ => stay,
    }
}

fn chosen(entry: usize, sources: &[String], groups: &[String]) -> Option<(View, ViewResponse)> {
    if let Some(source) = sources.get(entry) {
        let streamed = View::Streamed { group: None };
        return Some((streamed, ViewResponse::Switch(source.clone())));
    }
    let group = match entry - sources.len() {
        _ if groups.is_empty() => return None,
        0 => None,
        index => Some(groups.get(index - 1)?.clone()),
    };
    Some((View::List { group }, ViewResponse::CloseChooser))
}

#[cfg(test)]
mod tests;
