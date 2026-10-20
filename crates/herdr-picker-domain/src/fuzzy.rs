#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match<'t, T> {
    pub item: &'t T,
    pub matched: Vec<usize>,
}

pub trait Matcher {
    fn rank<'t, T>(
        &self,
        query: &str,
        items: &'t [T],
        label: impl Fn(&T) -> &str,
    ) -> Vec<Match<'t, T>>;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Fuzzy;

impl Matcher for Fuzzy {
    fn rank<'t, T>(
        &self,
        query: &str,
        items: &'t [T],
        label: impl Fn(&T) -> &str,
    ) -> Vec<Match<'t, T>> {
        let mut hits: Vec<_> = items
            .iter()
            .filter_map(|item| Some((find(query, label(item))?, item)))
            .collect();
        hits.sort_by_key(|((score, _), _)| *score);
        hits.into_iter()
            .map(|((_, matched), item)| Match { item, matched })
            .collect()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Score {
    span: usize,
    start: usize,
    length: usize,
}

fn find(query: &str, label: &str) -> Option<(Score, Vec<usize>)> {
    let query: Vec<char> = query.chars().collect();
    let label: Vec<char> = label.chars().collect();
    let Some((first, rest)) = query.split_first() else {
        return Some((Score::default(), Vec::new()));
    };
    (0..label.len())
        .filter(|&start| same_letter(label[start], *first))
        .map_while(|start| positions_from(start, rest, &label))
        .map(|positions| {
            let score = Score {
                span: positions.last().map_or(0, |end| end - positions[0] + 1),
                start: positions[0],
                length: label.len(),
            };
            (score, positions)
        })
        .min_by_key(|(score, _)| *score)
}

fn positions_from(start: usize, rest: &[char], label: &[char]) -> Option<Vec<usize>> {
    let mut positions = vec![start];
    let mut end = start;
    for wanted in rest {
        end += 1 + label
            .get(end + 1..)?
            .iter()
            .position(|found| same_letter(*found, *wanted))?;
        positions.push(end);
    }
    Some(positions)
}

fn same_letter(found: char, wanted: char) -> bool {
    found.to_lowercase().eq(wanted.to_lowercase())
}

#[cfg(test)]
mod tests;
