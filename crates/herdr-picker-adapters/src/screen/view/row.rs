use super::layout::{BEFORE_LABEL, LABEL_INDENT, Layout};
use super::palette::Palette;
use super::text::{Span, cut, line, room, span, width, width_of};
use herdr_picker_domain::{Frame, Row, RowLook};

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
    let palette = layout.palette();
    let mut spans = vec![
        key_cell(row.target.pick_key, searching, palette),
        span("", LABEL_INDENT),
    ];
    spans.extend(name(row, palette));
    spans.push(filler(
        layout.row_look(),
        layout
            .directory_start()
            .saturating_sub(BEFORE_LABEL + width_of(&row.target.label)),
        palette,
    ));
    let mut spans = cut(spans, room(width, 0), "");
    let left = room(width, self::width(&spans));
    spans.extend(cut(
        vec![span(palette.faint, row.target.directory.as_str())],
        left,
        palette.faint,
    ));
    let background = background(row.selected, index, layout.row_look(), palette);
    line(spans, background, palette.text, width)
}

fn background(selected: bool, index: usize, look: RowLook, palette: &Palette) -> &'static str {
    if selected {
        palette.bar
    } else if look.stripes && index % 2 == 1 {
        palette.stripe
    } else {
        palette.base
    }
}

fn filler(look: RowLook, filler: usize, palette: &Palette) -> Span {
    if look.leaders {
        let dots = "·".repeat(filler.saturating_sub(2));
        span(palette.leader, format!(" {dots} "))
    } else {
        span("", " ".repeat(filler))
    }
}

fn key_cell(key: char, searching: bool, palette: &Palette) -> Span {
    if searching {
        span("", format!("{:>KEY_COLUMN_WIDTH$}", ""))
    } else {
        span(palette.key, format!("{key:>KEY_COLUMN_WIDTH$}"))
    }
}

fn name(row: &Row<'_>, palette: &Palette) -> Vec<Span> {
    let plain = if row.open { palette.open } else { "" };
    row.target
        .label
        .chars()
        .enumerate()
        .map(|(at, character)| {
            let style = if row.matched.contains(&at) {
                palette.matched
            } else {
                plain
            };
            span(style, character)
        })
        .collect()
}
