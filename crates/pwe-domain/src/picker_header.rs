use crate::picker_view::{EVERY_GROUP, View};

// Without sources, a picker's groups are what the chooser offers, so they read as its source.
pub fn header(titles: &[String], view: &View, source: Option<&str>, groups: &[String]) -> String {
    let title = titles.join(" › ");
    let group = match view {
        View::Chooser { .. } => return format!("{title} › Choose source"),
        View::List { group: Some(group) } => group.as_str(),
        _ => EVERY_GROUP,
    };
    let source_part = source.map(|source| format!("Source: {source}"));
    let group_part = (!groups.is_empty()).then(|| match source {
        Some(_) => format!("Group: {group}"),
        None => format!("Source: {group}"),
    });
    std::iter::once(title)
        .chain(source_part)
        .chain(group_part)
        .collect::<Vec<_>>()
        .join(" | ")
}

pub fn title(name: &str) -> String {
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

#[cfg(test)]
mod tests;
