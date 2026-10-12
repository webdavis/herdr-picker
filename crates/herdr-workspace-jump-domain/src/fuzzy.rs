pub trait Matcher {
    fn rank<'t, T>(&self, query: &str, items: &'t [T], label: impl Fn(&T) -> &str) -> Vec<&'t T>;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Fuzzy;

impl Matcher for Fuzzy {
    fn rank<'t, T>(&self, query: &str, items: &'t [T], label: impl Fn(&T) -> &str) -> Vec<&'t T> {
        items
            .iter()
            .filter(|item| holds_in_order(label(item), query))
            .collect()
    }
}

fn holds_in_order(label: &str, query: &str) -> bool {
    let mut letters = label.chars().flat_map(char::to_lowercase);
    query
        .chars()
        .flat_map(char::to_lowercase)
        .all(|wanted| letters.any(|letter| letter == wanted))
}

#[cfg(test)]
mod tests;
