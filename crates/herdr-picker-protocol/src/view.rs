use herdr_picker_domain::{View, ViewState};

// Line one is the view's mode, group and source, tab-separated; line two the description; line
// three the sources, tab-separated; each later line a title.
pub fn encode_view_state(state: &ViewState) -> String {
    let (mode, group) = match &state.view {
        View::Streamed { group } => ("streamed", group),
        View::List { group } => ("list", group),
        View::Chooser { group } => ("chooser", group),
    };
    let one_line = |text: &str| text.replace('\n', " ");
    let first = format!(
        "{mode}\t{}\t{}",
        group.as_deref().unwrap_or_default(),
        state.source.as_deref().unwrap_or_default()
    );
    [
        first,
        one_line(&state.description),
        state.sources.join("\t"),
    ]
    .into_iter()
    .chain(state.titles.iter().map(|title| one_line(title)))
    .collect::<Vec<_>>()
    .join("\n")
}

pub fn parse_view_state(body: &str) -> ViewState {
    let mut lines = body.lines();
    let mut fields = lines.next().unwrap_or_default().split('\t');
    let mut next_field = || Some(fields.next()?.to_string()).filter(|field| !field.is_empty());
    let (mode, group, source) = (next_field(), next_field(), next_field());
    let view = match mode.as_deref().unwrap_or_default() {
        "list" => View::List { group },
        "chooser" => View::Chooser { group },
        _ => View::Streamed { group },
    };
    let description = lines.next().unwrap_or_default().to_string();
    let sources = lines.next().unwrap_or_default();
    ViewState {
        view,
        description,
        source,
        sources: sources
            .split('\t')
            .filter(|source| !source.is_empty())
            .map(str::to_string)
            .collect(),
        titles: lines.map(str::to_string).collect(),
    }
}

#[cfg(test)]
mod tests;
