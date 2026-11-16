use frizbee::{Config, Matcher};
use pwe_domain::finder::RowMatcher;

pub struct FrizbeeMatcher;

impl RowMatcher for FrizbeeMatcher {
    fn rank(&self, query: &str, rows: &[String]) -> Vec<usize> {
        Matcher::from_query(query, &Config::default())
            .match_list(rows)
            .into_iter()
            .map(|found| found.index as usize)
            .collect()
    }

    fn highlight(&self, query: &str, row: &str) -> Vec<usize> {
        Matcher::from_query(query, &Config::default())
            .match_one_indices(row, 0)
            .map(|found| characters_in_reading_order(row, &found.indices))
            .unwrap_or_default()
    }
}

fn characters_in_reading_order(row: &str, matched_bytes: &[u32]) -> Vec<usize> {
    row.char_indices()
        .enumerate()
        .filter(|(_, (byte, _))| {
            matched_bytes
                .iter()
                .any(|&matched| matched as usize == *byte)
        })
        .map(|(character, _)| character)
        .collect()
}

#[cfg(test)]
mod tests;
