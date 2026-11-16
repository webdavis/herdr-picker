use super::{Finder, RowMatcher};
use crate::PickerRow;

impl<M: RowMatcher> Finder<M> {
    pub(super) fn move_cursor(&mut self, by: isize) {
        let Some(place) = self
            .cursor
            .and_then(|row| self.ranked.iter().position(|&ranked| ranked == row))
        else {
            return;
        };
        let last = self.ranked.len() - 1;
        let place = place.saturating_add_signed(by).min(last);
        self.cursor = Some(self.ranked[place]);
    }

    pub(super) fn mark_and_move(&mut self, by: isize) {
        let Some(row) = self.cursor.filter(|_| !self.in_chooser()) else {
            return;
        };
        match self.marks.iter().position(|&marked| marked == row) {
            Some(place) => {
                self.marks.remove(place);
            }
            None => self.marks.push(row),
        }
        self.move_cursor(by);
    }

    pub(super) fn chosen(&self) -> Vec<usize> {
        match (self.marks.is_empty(), self.in_chooser()) {
            (false, _) => self.marks.clone(),
            (true, false) => self.cursor.into_iter().collect(),
            (true, true) => Vec::new(),
        }
    }

    pub(super) fn chosen_rows(&self) -> Vec<&PickerRow> {
        let chosen = self.chosen();
        chosen
            .iter()
            .filter_map(|&row| self.rows.get(row))
            .collect()
    }
}
