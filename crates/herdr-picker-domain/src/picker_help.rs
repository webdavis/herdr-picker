use crate::picker_header::title;
use crate::picker_row::{Action, HerdrCall, PickerRow};

const HELP_KEY: &str = "F1 help";
const CHOOSE: (&str, &str) = ("ctrl-space", "choose a source");
const GENERAL: [(&str, &str); 5] = [
    ("tab", "mark a row; a key then acts on every marked row"),
    (
        "ctrl-q",
        "save the marked rows' files and lines as the quickfix list",
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
    row.offered()
        .filter_map(|(key, action)| verb(action).map(|verb| format!("{} {verb}", key.label())))
        .chain(std::iter::once(HELP_KEY.to_string()))
        .collect()
}

// Each key the row claims, then the keys every picker has.
pub fn help_entries(row: Option<&PickerRow>, grouped: bool) -> Vec<(String, String)> {
    let claimed = row.into_iter().flat_map(|row| {
        row.offered().filter_map(|(key, action)| {
            describe(action).map(|text| (key.label().to_string(), text))
        })
    });
    let general = grouped
        .then_some(CHOOSE)
        .into_iter()
        .chain(GENERAL)
        .map(|(key, text)| (key.to_string(), text.to_string()));
    claimed.chain(general).collect()
}

fn describe(action: &Action) -> Option<String> {
    let verb = verb(action)?;
    Some(match detail(action) {
        Some(detail) => format!("{verb} {detail}"),
        None => verb.to_string(),
    })
}

fn verb(action: &Action) -> Option<&'static str> {
    Some(match action {
        Action::Insert(_) => "insert",
        Action::Copy(_) => "copy",
        Action::CopyStay(_) => "copy and stay",
        Action::Cd(_) => "cd",
        Action::Run(_) => "run",
        Action::Edit(_) => "edit",
        Action::Open(_) => "open",
        Action::Next(_) => "go to",
        Action::Confirm { .. } => "confirm",
        Action::Source(_) => "show",
        Action::State(_) => "switch",
        Action::Reload => "reload",
        Action::Back => "back",
        Action::InPane { .. } => "open in a new pane",
        Action::Herdr(HerdrCall::Prompt { .. }) => "send",
        Action::Herdr(HerdrCall::CloseTab(_)) => "close",
        Action::Herdr(_) => "focus",
        Action::Nothing => return None,
    })
}

fn detail(action: &Action) -> Option<String> {
    match action {
        Action::Insert(text)
        | Action::Copy(text)
        | Action::CopyStay(text)
        | Action::Cd(text)
        | Action::Run(text)
        | Action::Open(text)
        | Action::Source(text) => Some(text.clone()),
        Action::Edit(locations) => Some(
            locations
                .iter()
                .map(|location| match location.line {
                    Some(line) => format!("{}:{line}", location.path),
                    None => location.path.clone(),
                })
                .collect::<Vec<_>>()
                .join(" "),
        ),
        Action::Next(call) => Some(title(&call.name)),
        Action::Confirm { question, .. } => Some(question.clone()),
        Action::InPane { action, .. } => detail(action),
        Action::Herdr(
            HerdrCall::FocusPane(id)
            | HerdrCall::FocusTab(id)
            | HerdrCall::FocusWorkspace(id)
            | HerdrCall::CloseTab(id)
            | HerdrCall::Prompt { pane_id: id, .. },
        ) => Some(id.clone()),
        Action::State(_) | Action::Reload | Action::Back | Action::Nothing => None,
    }
}

#[cfg(test)]
mod tests;
