use crate::picker_header::title;
use crate::picker_row::{Action, HerdrCall};

pub(super) fn describe(action: &Action) -> Option<String> {
    let verb = verb(action)?;
    Some(match detail(action) {
        Some(detail) => format!("{verb} {detail}"),
        None => verb.to_string(),
    })
}

pub(super) fn verb(action: &Action) -> Option<&'static str> {
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

pub(super) fn detail(action: &Action) -> Option<String> {
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
