pub trait Matcher {
    fn rank<'t, T>(&self, query: &str, items: &'t [T], label: impl Fn(&T) -> &str) -> Vec<&'t T>;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Fuzzy;

impl Matcher for Fuzzy {
    fn rank<'t, T>(&self, query: &str, items: &'t [T], label: impl Fn(&T) -> &str) -> Vec<&'t T> {
        let mut matches: Vec<_> = items
            .iter()
            .filter_map(|item| Some((score(query, label(item))?, item)))
            .collect();
        matches.sort_by_key(|&(score, _)| score);
        matches.into_iter().map(|(_, item)| item).collect()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Score {
    span: usize,
    start: usize,
    length: usize,
}

fn score(query: &str, label: &str) -> Option<Score> {
    let query = lowercase(query);
    let label = lowercase(label);
    let Some((first, rest)) = query.split_first() else {
        return Some(Score::default());
    };
    let (span, start) = (0..label.len())
        .filter(|&start| label[start] == *first)
        .map_while(|start| Some((span_from(start, rest, &label)?, start)))
        .min()?;
    Some(Score {
        span,
        start,
        length: label.len(),
    })
}

fn span_from(start: usize, rest: &[char], label: &[char]) -> Option<usize> {
    let mut end = start;
    for wanted in rest {
        end += 1 + label
            .get(end + 1..)?
            .iter()
            .position(|found| found == wanted)?;
    }
    Some(end - start + 1)
}

fn lowercase(text: &str) -> Vec<char> {
    text.chars().flat_map(char::to_lowercase).collect()
}

#[cfg(test)]
mod tests;
