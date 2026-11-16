use super::{Finder, Found, RowMatcher};
use crate::{PickerRow, Preview, QuickfixEntry, ViewKey};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input {
    Key(String),
    RowsArrived(Vec<PickerRow>),
    RowsEnded,
    RowsFailed(String),
    HeaderArrived(String),
    PreviewArrived {
        row: usize,
        text: String,
    },
    Resized {
        list_height: usize,
        preview_width: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    Draw,
    ShowPreview {
        row: usize,
        preview: Preview,
        width: usize,
    },
    Copy(String),
    SaveQuickfix(Vec<QuickfixEntry>),
    Close(Found),
    CloseWithoutFinding,
}

pub fn step<M: RowMatcher>(mut finder: Finder<M>, input: Input) -> (Finder<M>, Vec<Effect>) {
    let previewed = finder.previewed_row();
    let resized = matches!(input, Input::Resized { .. });
    let mut effects = match input {
        Input::Key(key) => finder.press(&key),
        Input::RowsArrived(rows) => {
            finder.arrive(rows);
            vec![Effect::Draw]
        }
        Input::RowsEnded => finder.end_rows(None),
        Input::RowsFailed(error) => finder.end_rows(Some(error)),
        Input::HeaderArrived(line) => {
            finder.header_line = Some(line);
            vec![Effect::Draw]
        }
        Input::PreviewArrived { row, text } => {
            finder.preview = (finder.cursor == Some(row)).then_some((row, text));
            vec![Effect::Draw]
        }
        Input::Resized {
            list_height,
            preview_width,
        } => {
            finder.list_height = list_height;
            finder.preview_width = preview_width;
            vec![Effect::Draw]
        }
    };
    if resized || finder.previewed_row() != previewed {
        effects.extend(finder.preview_request());
    }
    (finder, effects)
}

impl<M: RowMatcher> Finder<M> {
    fn end_rows(&mut self, error: Option<String>) -> Vec<Effect> {
        self.rows_ended = true;
        self.run_error = error;
        self.respond(ViewKey::Loaded, None);
        vec![Effect::Draw]
    }

    fn previewed_row(&self) -> Option<usize> {
        self.cursor.filter(|_| !self.in_chooser())
    }

    fn preview_request(&self) -> Option<Effect> {
        let row = self.previewed_row()?;
        Some(Effect::ShowPreview {
            row,
            preview: self.rows.get(row)?.preview.clone()?,
            width: self.preview_width,
        })
    }
}

#[cfg(test)]
mod tests;
