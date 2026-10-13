#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match<'t, T> {
    pub item: &'t T,
    pub positions: Vec<usize>,
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
        let mut found: Vec<_> = items
            .iter()
            .filter_map(|item| Some((find(query, label(item))?, item)))
            .collect();
        found.sort_by_key(|(found, _)| found.score);
        found
            .into_iter()
            .map(|(found, item)| Match {
                item,
                positions: found.positions,
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Score {
    span: usize,
    start: usize,
    length: usize,
}

struct Found {
    score: Score,
    positions: Vec<usize>,
}

fn find(query: &str, label: &str) -> Option<Found> {
    let query = lowercase(query);
    let label = lowercase(label);
    let Some((first, rest)) = query.split_first() else {
        return Some(Found {
            score: Score::default(),
            positions: Vec::new(),
        });
    };
    (0..label.len())
        .filter(|&start| label[start] == *first)
        .map_while(|start| positions_from(start, rest, &label))
        .map(|positions| Found {
            score: Score {
                span: positions.last().map_or(0, |end| end - positions[0] + 1),
                start: positions[0],
                length: label.len(),
            },
            positions,
        })
        .min_by_key(|found| found.score)
}

fn positions_from(start: usize, rest: &[char], label: &[char]) -> Option<Vec<usize>> {
    let mut positions = vec![start];
    let mut end = start;
    for wanted in rest {
        end += 1 + label
            .get(end + 1..)?
            .iter()
            .position(|found| found == wanted)?;
        positions.push(end);
    }
    Some(positions)
}

fn lowercase(text: &str) -> Vec<char> {
    text.chars().flat_map(char::to_lowercase).collect()
}

#[cfg(test)]
mod tests;
