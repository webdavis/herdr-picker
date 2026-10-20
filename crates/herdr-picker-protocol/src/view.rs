use herdr_picker_domain::{View, ViewState};

// Line one is the view as mode, tab, group; line two the description; each later line a title.
pub fn encode_view_state(state: &ViewState) -> String {
    let (mode, group) = match &state.view {
        View::Streamed { group } => ("streamed", group),
        View::List { group } => ("list", group),
        View::Chooser { group } => ("chooser", group),
    };
    let one_line = |text: &str| text.replace('\n', " ");
    std::iter::once(format!("{mode}\t{}", group.as_deref().unwrap_or_default()))
        .chain(std::iter::once(one_line(&state.description)))
        .chain(state.titles.iter().map(|title| one_line(title)))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn parse_view_state(body: &str) -> ViewState {
    let mut lines = body.lines();
    let view_line = lines.next().unwrap_or_default();
    let (mode, group) = view_line.split_once('\t').unwrap_or((view_line, ""));
    let group = Some(group.to_string()).filter(|group| !group.is_empty());
    let view = match mode {
        "list" => View::List { group },
        "chooser" => View::Chooser { group },
        _ => View::Streamed { group },
    };
    ViewState {
        view,
        description: lines.next().unwrap_or_default().to_string(),
        titles: lines.map(str::to_string).collect(),
    }
}

#[cfg(test)]
mod tests;
