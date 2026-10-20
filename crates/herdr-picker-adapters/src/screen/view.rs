use herdr_picker_domain::{Frame, Legend, Refusal, SearchLine};

mod layout;
mod palette;
mod row;
mod text;

pub use layout::Layout;
pub(crate) use palette::{Palette, palette};
use row::workspace_rows;
use text::{cut, line, room, span, width_of};

const MARGIN: &str = " ";
const SEARCH_PROMPT: &str = " /";
const MIN_GAP_BEFORE_COUNT: usize = 2;
const ALREADY_CURRENT: &str = "already in ";
const TYPE_TO_FILTER: &str = " type to filter";
const PROMPT_ROW: usize = 1;
const FIRST_COLUMN: usize = 1;

pub(super) fn render(frame: &Frame<'_>, layout: &Layout, width: usize, height: usize) -> String {
    let legend = layout.legend();
    let palette = layout.palette();
    let above = [
        prompt_row(frame, legend, palette, width),
        blank(palette, width),
    ];
    let below = [
        message_row(frame, layout.notice(), palette, width),
        faint_line(legend.keys, palette, width),
    ];
    let room = height
        .checked_sub(above.len() + below.len())
        .unwrap_or(frame.rows.len());
    let mut workspaces = workspace_rows(frame, layout, room, width);
    workspaces.resize(room, blank(palette, width));
    let mut drawn = [above.as_slice(), &workspaces, &below].concat().join("\n");
    if let Some(search) = &frame.search {
        drawn.push_str(&cursor_in_prompt(search));
    }
    drawn
}

fn refusal_text(label: &str) -> String {
    format!("{ALREADY_CURRENT}{label}")
}

fn message_row(frame: &Frame<'_>, notice: Option<&str>, palette: &Palette, width: usize) -> String {
    match (frame.refusal, notice) {
        (Some(Refusal::AlreadyCurrent(target)), _) => margin_line(
            palette.refusal,
            &refusal_text(&target.label),
            palette,
            width,
        ),
        (None, Some(notice)) => faint_line(notice, palette, width),
        (None, None) => blank(palette, width),
    }
}

fn prompt_row(frame: &Frame<'_>, legend: Legend, palette: &Palette, width: usize) -> String {
    let Some(search) = &frame.search else {
        return faint_line(legend.search, palette, width);
    };
    let typed = format!("{SEARCH_PROMPT}{}", search.text);
    let count = format!("{}/{}{MARGIN}", frame.rows.len(), frame.total);
    let hint = if search.text.is_empty() {
        TYPE_TO_FILTER
    } else {
        ""
    };
    let hint = cut(
        vec![span(palette.faint, hint)],
        room(
            width,
            width_of(&typed) + width_of(&count) + MIN_GAP_BEFORE_COUNT,
        ),
        palette.faint,
    );
    let gap = width
        .saturating_sub(width_of(&typed) + text::width(&hint) + width_of(&count))
        .max(MIN_GAP_BEFORE_COUNT);
    let mut spans = vec![span("", typed)];
    spans.extend(hint);
    spans.push(span("", " ".repeat(gap)));
    spans.push(span(palette.faint, count));
    line(spans, palette.base, palette.text, width)
}

fn cursor_in_prompt(search: &SearchLine) -> String {
    let column = FIRST_COLUMN + width_of(SEARCH_PROMPT) + search.cursor;
    format!("\x1b[{PROMPT_ROW};{column}H")
}

fn faint_line(text: &str, palette: &Palette, width: usize) -> String {
    margin_line(palette.faint, text, palette, width)
}

fn margin_line(style: &'static str, text: &str, palette: &Palette, width: usize) -> String {
    let spans = cut(
        vec![span(style, format!("{MARGIN}{text}"))],
        room(width, 0),
        style,
    );
    line(spans, palette.base, palette.text, width)
}

fn blank(palette: &Palette, width: usize) -> String {
    line(Vec::new(), palette.base, palette.text, width)
}

#[cfg(test)]
mod tests;
