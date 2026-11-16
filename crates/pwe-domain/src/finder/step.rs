use super::{Finder, Found, RowMatcher};
use crate::{PickerRow, QuickfixEntry, ViewKey};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input {
    Key(String),
    RowsArrived(Vec<PickerRow>),
    RowsEnded,
    PreviewArrived { row: usize, text: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    Draw,
    Copy(String),
    SaveQuickfix(Vec<QuickfixEntry>),
    Close(Found),
    CloseWithoutFinding,
}

pub fn step<M: RowMatcher>(mut finder: Finder<M>, input: Input) -> (Finder<M>, Vec<Effect>) {
    let effects = match input {
        Input::Key(key) => finder.press(&key),
        Input::RowsArrived(rows) => {
            finder.arrive(rows);
            vec![Effect::Draw]
        }
        Input::RowsEnded => {
            finder.rows_ended = true;
            finder.respond(ViewKey::Loaded, None);
            vec![Effect::Draw]
        }
        Input::PreviewArrived { row, text } => {
            finder.preview = (finder.cursor == Some(row)).then_some((row, text));
            vec![Effect::Draw]
        }
    };
    (finder, effects)
}

#[cfg(test)]
mod tests;
