use super::{Finder, RowMatcher};
use crate::{PickerRow, TextField};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListLine<'a> {
    pub label: &'a str,
    pub row: Option<&'a PickerRow>,
    pub matched: Vec<usize>,
    pub marked: bool,
    pub current: bool,
}

impl<M: RowMatcher> Finder<M> {
    pub fn visible(&self, height: usize) -> Vec<ListLine<'_>> {
        let place = self
            .cursor
            .and_then(|cursor| self.ranked.iter().position(|&ranked| ranked == cursor))
            .unwrap_or_default();
        let first = (place + 1).saturating_sub(height);
        let query = self.query.text();
        self.ranked
            .iter()
            .skip(first)
            .take(height)
            .map(|&candidate| self.list_line(candidate, &query))
            .collect()
    }

    fn list_line(&self, candidate: usize, query: &str) -> ListLine<'_> {
        let chooser = self.in_chooser();
        let row = (!chooser).then(|| &self.rows[candidate]);
        ListLine {
            label: match row {
                Some(row) => &row.label,
                None => &self.entries[candidate],
            },
            row,
            matched: self.matcher.highlight(query, &self.candidates()[candidate]),
            marked: !chooser && self.marks.contains(&candidate),
            current: self.cursor == Some(candidate),
        }
    }

    pub fn rows(&self) -> &[PickerRow] {
        &self.rows
    }

    pub fn match_count(&self) -> usize {
        self.ranked.len()
    }

    pub fn current_row(&self) -> Option<&PickerRow> {
        self.cursor
            .filter(|_| !self.in_chooser())
            .and_then(|row| self.rows.get(row))
    }

    pub fn preview(&self) -> Option<&str> {
        self.preview
            .as_ref()
            .filter(|(row, _)| self.cursor == Some(*row) && !self.in_chooser())
            .map(|(_, text)| text.as_str())
    }
}

#[cfg(test)]
mod tests;
