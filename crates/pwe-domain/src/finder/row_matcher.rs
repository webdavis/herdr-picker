use crate::{Fuzzy, Matcher};

pub trait RowMatcher {
    fn rank(&self, query: &str, rows: &[String]) -> Vec<usize>;
    fn highlight(&self, query: &str, row: &str) -> Vec<usize>;
}

impl RowMatcher for Fuzzy {
    fn rank(&self, query: &str, rows: &[String]) -> Vec<usize> {
        let numbered: Vec<(usize, &str)> = rows.iter().map(String::as_str).enumerate().collect();
        Matcher::rank(self, query, &numbered, |(_, row)| row)
            .into_iter()
            .map(|hit| hit.item.0)
            .collect()
    }

    fn highlight(&self, query: &str, row: &str) -> Vec<usize> {
        Matcher::rank(self, query, &[row], |row| row)
            .into_iter()
            .next()
            .map(|hit| hit.matched)
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests;
