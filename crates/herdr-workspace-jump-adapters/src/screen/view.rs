use herdr_workspace_jump_domain::{Frame, Legend, Refusal, SearchLine};

mod catppuccin_mocha;
mod layout;
mod row;
mod text;

use catppuccin_mocha::{BASE, FAINT, REFUSAL};
pub use layout::Layout;
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
    let above = [prompt_row(frame, legend, width), blank(width)];
    let below = [
        message_row(frame, layout.notice(), width),
        faint_line(legend.keys, width),
    ];
    let room = height
        .checked_sub(above.len() + below.len())
        .unwrap_or(frame.rows.len());
    let mut workspaces = workspace_rows(frame, layout, room, width);
    workspaces.resize(room, blank(width));
    let mut drawn = [above.as_slice(), &workspaces, &below].concat().join("\n");
    if let Some(search) = &frame.search {
        drawn.push_str(&cursor_in_prompt(search));
    }
    drawn
}

fn refusal_text(label: &str) -> String {
    format!("{ALREADY_CURRENT}{label}")
}

fn message_row(frame: &Frame<'_>, notice: Option<&str>, width: usize) -> String {
    match (frame.refusal, notice) {
        (Some(Refusal::AlreadyCurrent(target)), _) => {
            margin_line(REFUSAL, &refusal_text(&target.label), width)
        }
        (None, Some(notice)) => faint_line(notice, width),
        (None, None) => blank(width),
    }
}

fn prompt_row(frame: &Frame<'_>, legend: Legend, width: usize) -> String {
    let Some(search) = &frame.search else {
        return faint_line(legend.search, width);
    };
    let typed = format!("{SEARCH_PROMPT}{}", search.text);
    let count = format!("{}/{}{MARGIN}", frame.rows.len(), frame.total);
    let hint = if search.text.is_empty() {
        TYPE_TO_FILTER
    } else {
        ""
    };
    let hint = cut(
        vec![span(FAINT, hint)],
        room(
            width,
            width_of(&typed) + width_of(&count) + MIN_GAP_BEFORE_COUNT,
        ),
        FAINT,
    );
    let gap = width
        .saturating_sub(width_of(&typed) + text::width(&hint) + width_of(&count))
        .max(MIN_GAP_BEFORE_COUNT);
    let mut spans = vec![span("", typed)];
    spans.extend(hint);
    spans.push(span("", " ".repeat(gap)));
    spans.push(span(FAINT, count));
    line(spans, BASE, width)
}

fn cursor_in_prompt(search: &SearchLine) -> String {
    let column = FIRST_COLUMN + width_of(SEARCH_PROMPT) + search.cursor;
    format!("\x1b[{PROMPT_ROW};{column}H")
}

fn faint_line(text: &str, width: usize) -> String {
    margin_line(FAINT, text, width)
}

fn margin_line(style: &'static str, text: &str, width: usize) -> String {
    let spans = cut(
        vec![span(style, format!("{MARGIN}{text}"))],
        room(width, 0),
        style,
    );
    line(spans, BASE, width)
}

fn blank(width: usize) -> String {
    line(Vec::new(), BASE, width)
}

#[cfg(test)]
mod tests;
