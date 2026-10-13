use super::text::{fill, fit, room, width_of};
use super::{BOLD_BLUE, FAINT, GREY_BAR, MARGIN, RESET};
use herdr_workspace_jump_domain::{Frame, JumpTarget, Row};

const KEY_COLUMN_WIDTH: usize = 3;
const LABEL_INDENT: &str = "  ";
const GAP_BEFORE_DIRECTORY: &str = "  ";
const BEFORE_LABEL: usize = MARGIN.len() + KEY_COLUMN_WIDTH;

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

    fn name_column(self) -> usize {
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
    let paint = Paint::for_row(row);
    let key = key_cell(row.target.pick_key, searching);
    let name = name_cell(&row.target.label, columns, width);
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
        "{}{MARGIN}{BOLD_BLUE}{key}{RESET}{}{}{}",
        paint.bar,
        paint.cell("", &name),
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

fn name_cell(label: &str, columns: Columns, width: usize) -> String {
    let name = fit(&format!("{LABEL_INDENT}{label}"), room(width, BEFORE_LABEL));
    let column = room(width, BEFORE_LABEL).map_or(columns.name_column(), |left| {
        columns.name_column().min(left)
    });
    fill(&name, column)
}

struct Paint {
    bar: &'static str,
}

impl Paint {
    fn for_row(row: &Row<'_>) -> Self {
        Self {
            bar: if row.selected { GREY_BAR } else { "" },
        }
    }

    fn cell(&self, style: &str, text: &str) -> String {
        if text.is_empty() || (self.bar.is_empty() && style.is_empty()) {
            text.to_string()
        } else {
            format!("{}{style}{text}{RESET}", self.bar)
        }
    }
}
