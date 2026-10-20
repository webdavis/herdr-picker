use super::{Frame, Menu, Offered, Row, SearchLine};
use crate::field::TextField;
use crate::fuzzy::{Match, Matcher};

// What the menu shows: the offered workspaces, ranked against the search while there is one.
impl<'a, M: Matcher, F: TextField> Menu<'a, M, F> {
    pub fn frame(&self) -> Frame<'a> {
        let rows = self
            .shown()
            .into_iter()
            .enumerate()
            .map(|(row, hit)| Row {
                target: hit.item.target,
                selected: row == self.selected_row,
                open: hit.item.open,
                matched: hit.matched,
            })
            .collect();
        Frame {
            search: self.searching.then(|| SearchLine {
                text: self.field.text(),
                cursor: self.field.cursor(),
            }),
            rows,
            total: self.offered.len(),
            refusal: self.refusal,
        }
    }

    pub(super) fn shown(&self) -> Vec<Match<'_, Offered<'a>>> {
        if self.searching {
            self.matcher
                .rank(&self.field.text(), &self.offered, |offered| {
                    &offered.target.label
                })
        } else {
            self.offered
                .iter()
                .map(|item| Match {
                    item,
                    matched: Vec::new(),
                })
                .collect()
        }
    }
}
