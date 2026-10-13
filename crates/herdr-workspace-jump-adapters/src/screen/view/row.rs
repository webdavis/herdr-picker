use super::layout::Layout;
use super::text::{Span, cut, pad, paint, room, span, width};
use super::{BOLD_BLUE, FAINT, GREEN, GREY_BAR};
use herdr_workspace_jump_domain::{Frame, Row};

const KEY_COLUMN_WIDTH: usize = 4;
const OPEN_MARK: &str = "  ● ";
const NO_MARK: &str = "    ";

pub(super) fn workspace_rows(
    frame: &Frame<'_>,
    layout: &Layout,
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
        .map(|row| workspace_row(row, searching, layout, width))
        .collect()
}

fn workspace_row(row: &Row<'_>, searching: bool, layout: &Layout, width: usize) -> String {
    let mut spans = vec![key_cell(row.target.pick_key, searching), mark(row.open)];
    spans.extend(lit_name(&row.target.label, &row.matched));
    let mut spans = cut(pad(spans, layout.directory_start()), room(width, 0), "");
    let left = room(width, self::width(&spans));
    spans.extend(cut(
        vec![span(FAINT, row.target.directory.as_str())],
        left,
        FAINT,
    ));
    if row.selected {
        paint(&pad(spans, width), GREY_BAR)
    } else {
        paint(&spans, "")
    }
}

fn key_cell(key: char, searching: bool) -> Span {
    if searching {
        span("", format!("{:>KEY_COLUMN_WIDTH$}", ""))
    } else {
        span(BOLD_BLUE, format!("{key:>KEY_COLUMN_WIDTH$}"))
    }
}

fn mark(open: bool) -> Span {
    if open {
        span(GREEN, OPEN_MARK)
    } else {
        span("", NO_MARK)
    }
}

fn lit_name(label: &str, matched: &[usize]) -> Vec<Span> {
    label
        .chars()
        .enumerate()
        .map(|(at, character)| {
            let style = if matched.contains(&at) { BOLD_BLUE } else { "" };
            span(style, character)
        })
        .collect()
}
