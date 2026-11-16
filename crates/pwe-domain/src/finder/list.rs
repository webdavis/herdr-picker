use super::{Ending, Found, Remark, RowMatcher};
use crate::look::unstyled;
use crate::{Edit, PickerRow, ReadlineField, TextField, View, ViewState};

pub struct Finder<M> {
    matcher: M,
    pub(super) query: ReadlineField,
    pub(super) rows: Vec<PickerRow>,
    plain: Vec<String>,
    pub(super) state: ViewState,
    pub(super) entries: Vec<String>,
    pub(super) ranked: Vec<usize>,
    pub(super) cursor: Option<usize>,
    pub(super) marks: Vec<usize>,
    pub(super) rows_ended: bool,
    pub(super) remark: Option<Remark>,
    pub(super) help_shown: bool,
}

impl<M: RowMatcher> Finder<M> {
    pub fn new(matcher: M) -> Self {
        Self {
            matcher,
            query: ReadlineField::default(),
            rows: Vec::new(),
            plain: Vec::new(),
            state: ViewState {
                view: View::Streamed { group: None },
                titles: Vec::new(),
                description: String::new(),
                source: None,
                sources: Vec::new(),
            },
            entries: Vec::new(),
            ranked: Vec::new(),
            cursor: None,
            marks: Vec::new(),
            rows_ended: false,
            remark: None,
            help_shown: false,
        }
    }

    pub fn with_view(self, state: ViewState) -> Self {
        Self { state, ..self }
    }

    pub fn listed(&self) -> Vec<&str> {
        let candidates = self.candidates();
        self.ranked
            .iter()
            .map(|&candidate| candidates[candidate].as_str())
            .collect()
    }

    pub fn rows_ended(&self) -> bool {
        self.rows_ended
    }

    pub fn query(&self) -> String {
        self.query.text()
    }

    pub fn remark(&self) -> Option<&Remark> {
        self.remark.as_ref()
    }

    pub fn help_shown(&self) -> bool {
        self.help_shown
    }

    pub fn view(&self) -> &ViewState {
        &self.state
    }

    pub(super) fn in_chooser(&self) -> bool {
        matches!(self.state.view, View::Chooser { .. })
    }

    fn candidates(&self) -> &[String] {
        match self.in_chooser() {
            true => &self.entries,
            false => &self.plain,
        }
    }

    pub(super) fn arrive(&mut self, rows: Vec<PickerRow>) {
        self.plain
            .extend(rows.iter().map(|row| unstyled(&row.label)));
        self.rows.extend(rows);
        self.rank();
        if !self.cursor.is_some_and(|row| self.ranked.contains(&row)) {
            self.cursor = self.ranked.first().copied();
        }
    }

    pub(super) fn rank(&mut self) {
        self.ranked = self.matcher.rank(&self.query.text(), self.candidates());
        if let View::List { group: Some(group) } = &self.state.view {
            let rows = &self.rows;
            self.ranked
                .retain(|&row| rows[row].group.as_ref() == Some(group));
        }
    }

    pub(super) fn restart_query(&mut self) {
        self.query = ReadlineField::default();
        self.rank();
        self.cursor = self.ranked.first().copied();
    }

    pub(super) fn edit(&mut self, edit: Edit) {
        let before = self.query.text();
        self.query.apply(edit);
        if self.query.text() != before {
            self.rank();
            self.cursor = self.ranked.first().copied();
        }
    }

    pub(super) fn type_text(&mut self, typed: &str) {
        let typed = if typed == "space" { " " } else { typed };
        let mut characters = typed.chars();
        if let (Some(character), None) = (characters.next(), characters.next()) {
            self.edit(Edit::Insert(character));
        }
    }

    pub(super) fn found(&self, ending: Ending) -> Found {
        Found {
            query: self.query.text(),
            ending,
            rows: self.chosen(),
        }
    }
}
