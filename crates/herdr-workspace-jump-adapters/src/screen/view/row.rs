use super::layout::{BEFORE_LABEL, LABEL_INDENT, Layout};
use super::text::{Span, cut, pad, paint, room, span, width, width_of};
use super::{BOLD_BLUE, FAINT, GREEN, GREY_BAR, LEADER, STRIPE};
use herdr_workspace_jump_domain::{Frame, Row, RowLook};

const KEY_COLUMN_WIDTH: usize = 4;

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
        .enumerate()
        .skip(top)
        .take(room)
        .map(|(index, row)| workspace_row(row, index, searching, layout, width))
        .collect()
}

fn workspace_row(
    row: &Row<'_>,
    index: usize,
    searching: bool,
    layout: &Layout,
    width: usize,
) -> String {
    let mut spans = vec![
        key_cell(row.target.pick_key, searching),
        span("", LABEL_INDENT),
    ];
    spans.extend(name(row));
    spans.push(filler(
        layout.row_look(),
        layout
            .directory_start()
            .saturating_sub(BEFORE_LABEL + width_of(&row.target.label)),
    ));
    let mut spans = cut(spans, room(width, 0), "");
    let left = room(width, self::width(&spans));
    spans.extend(cut(
        vec![span(FAINT, row.target.directory.as_str())],
        left,
        FAINT,
    ));
    let background = background(row.selected, index, layout.row_look());
    if background.is_empty() {
        paint(&spans, "")
    } else {
        paint(&pad(spans, width), background)
    }
}

fn background(selected: bool, index: usize, look: RowLook) -> &'static str {
    if selected {
        GREY_BAR
    } else if look.stripes && index % 2 == 1 {
        STRIPE
    } else {
        ""
    }
}

fn filler(look: RowLook, filler: usize) -> Span {
    if look.leaders {
        let dots = "·".repeat(filler.saturating_sub(2));
        span(LEADER, format!(" {dots} "))
    } else {
        span("", " ".repeat(filler))
    }
}

fn key_cell(key: char, searching: bool) -> Span {
    if searching {
        span("", format!("{:>KEY_COLUMN_WIDTH$}", ""))
    } else {
        span(BOLD_BLUE, format!("{key:>KEY_COLUMN_WIDTH$}"))
    }
}

fn name(row: &Row<'_>) -> Vec<Span> {
    let plain = if row.open { GREEN } else { "" };
    row.target
        .label
        .chars()
        .enumerate()
        .map(|(at, character)| {
            let style = if row.matched.contains(&at) {
                BOLD_BLUE
            } else {
                plain
            };
            span(style, character)
        })
        .collect()
}
