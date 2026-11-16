use super::{Ending, Found, RowMatcher};
use crate::look::unstyled;
use crate::{Edit, Key, ReadlineField, TextField};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input {
    Key(String),
    RowsArrived(Vec<String>),
    RowsEnded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    Draw,
    Close(Found),
}

pub struct Finder<M> {
    matcher: M,
    query: ReadlineField,
    plain: Vec<String>,
    ranked: Vec<usize>,
    cursor: Option<usize>,
    marks: Vec<usize>,
    rows_ended: bool,
}

impl<M: RowMatcher> Finder<M> {
    pub fn new(matcher: M) -> Self {
        Self {
            matcher,
            query: ReadlineField::default(),
            plain: Vec::new(),
            ranked: Vec::new(),
            cursor: None,
            marks: Vec::new(),
            rows_ended: false,
        }
    }

    pub fn listed(&self) -> Vec<&str> {
        self.ranked
            .iter()
            .map(|&row| self.plain[row].as_str())
            .collect()
    }

    pub fn rows_ended(&self) -> bool {
        self.rows_ended
    }

    fn press(&mut self, key: &str) -> Effect {
        match key {
            "enter" => return Effect::Close(self.found(Ending::Key(Key::enter()))),
            "tab" => self.mark_and_move(1),
            "shift-tab" => self.mark_and_move(-1),
            "down" => self.move_cursor(1),
            "up" => self.move_cursor(-1),
            typed => self.type_text(typed),
        }
        Effect::Draw
    }

    fn type_text(&mut self, typed: &str) {
        let typed = if typed == "space" { " " } else { typed };
        let mut characters = typed.chars();
        if let (Some(character), None) = (characters.next(), characters.next()) {
            self.query.apply(Edit::Insert(character));
            self.rank();
            self.cursor = self.ranked.first().copied();
        }
    }

    fn arrive(&mut self, rows: Vec<String>) {
        self.plain.extend(rows.iter().map(|row| unstyled(row)));
        self.rank();
        if !self.cursor.is_some_and(|row| self.ranked.contains(&row)) {
            self.cursor = self.ranked.first().copied();
        }
    }

    fn rank(&mut self) {
        self.ranked = self.matcher.rank(&self.query.text(), &self.plain);
    }

    fn move_cursor(&mut self, by: isize) {
        let Some(place) = self.cursor_place() else {
            return;
        };
        let last = self.ranked.len() - 1;
        let place = place.saturating_add_signed(by).min(last);
        self.cursor = Some(self.ranked[place]);
    }

    fn cursor_place(&self) -> Option<usize> {
        self.cursor
            .and_then(|row| self.ranked.iter().position(|&ranked| ranked == row))
    }

    fn mark_and_move(&mut self, by: isize) {
        let Some(row) = self.cursor else {
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

    fn found(&self, ending: Ending) -> Found {
        let rows = if self.marks.is_empty() {
            self.cursor.into_iter().collect()
        } else {
            self.marks.clone()
        };
        Found {
            query: self.query.text(),
            ending,
            rows,
        }
    }
}

pub fn step<M: RowMatcher>(mut finder: Finder<M>, input: Input) -> (Finder<M>, Vec<Effect>) {
    let effect = match input {
        Input::Key(key) => finder.press(&key),
        Input::RowsArrived(rows) => {
            finder.arrive(rows);
            Effect::Draw
        }
        Input::RowsEnded => {
            finder.rows_ended = true;
            Effect::Draw
        }
    };
    (finder, vec![effect])
}

#[cfg(test)]
mod tests;
