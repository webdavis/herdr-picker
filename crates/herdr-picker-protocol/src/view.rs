use herdr_picker_domain::View;

pub fn encode_view(view: &View) -> String {
    let (mode, group) = match view {
        View::Streamed => ("streamed", &None),
        View::List { group } => ("list", group),
        View::Chooser { group } => ("chooser", group),
    };
    format!("{mode}\t{}", group.as_deref().unwrap_or_default())
}

pub fn parse_view(body: &str) -> View {
    let (mode, group) = body.split_once('\t').unwrap_or((body, ""));
    let group = Some(group.to_string()).filter(|group| !group.is_empty());
    match mode {
        "list" => View::List { group },
        "chooser" => View::Chooser { group },
        _ => View::Streamed,
    }
}

#[cfg(test)]
mod tests;
