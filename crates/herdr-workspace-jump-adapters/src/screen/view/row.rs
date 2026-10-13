use super::name::{self, LABEL_INDENT};
use super::paint::Paint;
use super::text::{fill, fit, room, width_of};
use super::{BOLD_BLUE, FAINT, GREEN, MARGIN, RESET};
use herdr_workspace_jump_domain::{Frame, JumpTarget, Row};

const KEY_COLUMN_WIDTH: usize = 3;
const OPEN_MARK: &str = "  ●";
const NO_MARK: &str = "   ";
const MARK_COLUMN_WIDTH: usize = 3;
const GAP_BEFORE_DIRECTORY: &str = "  ";
pub(super) const BEFORE_LABEL: usize = MARGIN.len() + KEY_COLUMN_WIDTH + MARK_COLUMN_WIDTH;

#[derive(Clone, Copy, Debug)]
pub struct Columns {
    name: usize,
}

impl Columns {
    pub fn for_targets(targets: &[JumpTarget]) -> Self {
        Self {
            name: targets
                .iter()
                .map(|target| width_of(&target.label))
                .max()
                .unwrap_or_default(),
        }
    }

    pub(super) fn row_width(self, target: &JumpTarget) -> usize {
        BEFORE_LABEL + self.name_column() + width_of(&target.directory)
    }

    pub(super) fn name_column(self) -> usize {
        LABEL_INDENT.len() + self.name + GAP_BEFORE_DIRECTORY.len()
    }
}

pub(super) fn workspace_rows(
    frame: &Frame<'_>,
    columns: Columns,
    room: usize,
    width: usize,
) -> Vec<String> {
    let selected = frame.rows.iter().position(|row| row.selected).unwrap_or(0);
    let top = (selected + 1).saturating_sub(room);
    let searching = frame.search.is_some();
    frame
        .rows
        .iter()
        .skip(top)
        .take(room)
        .map(|row| workspace_row(row, searching, columns, width))
        .collect()
}

fn workspace_row(row: &Row<'_>, searching: bool, columns: Columns, width: usize) -> String {
    let paint = Paint::for_row(row.selected);
    let key = key_cell(row.target.pick_key, searching);
    let mark = if row.open { OPEN_MARK } else { NO_MARK };
    let name = name::padded(&row.target.label, columns, width);
    let directory = fit(
        &row.target.directory,
        room(width, BEFORE_LABEL + width_of(&name)),
    );
    let shown = BEFORE_LABEL + width_of(&name) + width_of(&directory);
    let pad = if row.selected {
        fill("", width.saturating_sub(shown))
    } else {
        String::new()
    };
    format!(
        "{}{MARGIN}{BOLD_BLUE}{key}{RESET}{}{}{}{}",
        paint.bar,
        paint.cell(if row.open { GREEN } else { "" }, mark),
        name::lit(&name, &row.matched, &paint),
        paint.cell(FAINT, &directory),
        paint.cell("", &pad)
    )
}

fn key_cell(key: char, searching: bool) -> String {
    let key = if searching {
        String::new()
    } else {
        key.to_string()
    };
    format!("{key:>KEY_COLUMN_WIDTH$}")
}
