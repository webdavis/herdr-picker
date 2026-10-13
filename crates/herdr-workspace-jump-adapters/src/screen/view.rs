use herdr_workspace_jump_domain::{Frame, JumpTarget, Legend, Refusal, SearchLine};

mod row;
mod text;

pub(super) use row::Columns;
use row::workspace_rows;
#[cfg(test)]
use text::ELLIPSIS;
use text::{fit, room, width_of};

const BOLD_BLUE: &str = "\x1b[1;34m";
const FAINT: &str = "\x1b[2m";
const GREEN: &str = "\x1b[32m";
const GREY_BAR: &str = "\x1b[48;5;236m";
const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";

const MARGIN: &str = " ";
const SEARCH_PROMPT: &str = " /";
const MIN_GAP_BEFORE_COUNT: usize = 2;
const ALREADY_CURRENT: &str = "already in ";
const TYPE_TO_FILTER: &str = " type to filter";
const PROMPT_ROW: usize = 1;
const FIRST_COLUMN: usize = 1;

pub(super) fn render(
    frame: &Frame<'_>,
    legend: Legend,
    columns: Columns,
    width: usize,
    height: usize,
) -> String {
    let above = [prompt_row(frame, legend, width), String::new()];
    let below = [refusal_row(frame, width), faint_line(legend.keys, width)];
    let room = height
        .checked_sub(above.len() + below.len())
        .unwrap_or(frame.rows.len());
    let mut workspaces = workspace_rows(frame, columns, room, width);
    workspaces.resize(room, String::new());
    let mut drawn = [above.as_slice(), &workspaces, &below].concat().join("\n");
    if let Some(search) = &frame.search {
        drawn.push_str(&cursor_in_prompt(search));
    }
    drawn
}

pub(super) fn columns_to_fit(targets: &[JumpTarget], legend: Legend) -> usize {
    let columns = Columns::for_targets(targets);
    let rows = targets.iter().map(|target| columns.row_width(target));
    let refusals = targets
        .iter()
        .map(|target| line_width(&refusal_text(&target.label)));
    let faint_lines = [legend.search, legend.keys].into_iter().map(line_width);
    rows.chain(refusals)
        .chain(faint_lines)
        .max()
        .unwrap_or_default()
}

fn line_width(text: &str) -> usize {
    MARGIN.len() + width_of(text)
}

fn refusal_text(label: &str) -> String {
    format!("{ALREADY_CURRENT}{label}")
}

fn refusal_row(frame: &Frame<'_>, width: usize) -> String {
    match frame.refusal {
        Some(Refusal::AlreadyCurrent(target)) => {
            let text = fit(&refusal_text(&target.label), room(width, MARGIN.len()));
            format!("{RED}{MARGIN}{text}{RESET}")
        }
        None => String::new(),
    }
}

fn prompt_row(frame: &Frame<'_>, legend: Legend, width: usize) -> String {
    let Some(search) = &frame.search else {
        return faint_line(legend.search, width);
    };
    let typed = format!("{SEARCH_PROMPT}{}", search.text);
    let count = format!("{}/{}{MARGIN}", frame.rows.len(), frame.total);
    let hint = if search.text.is_empty() {
        let left = width_of(&typed) + width_of(&count) + MIN_GAP_BEFORE_COUNT;
        fit(TYPE_TO_FILTER, room(width, left))
    } else {
        String::new()
    };
    let gap = width
        .saturating_sub(width_of(&typed) + width_of(&hint) + width_of(&count))
        .max(MIN_GAP_BEFORE_COUNT);
    format!(
        "{typed}{}{:gap$}{FAINT}{count}{RESET}",
        painted(FAINT, &hint),
        ""
    )
}

fn cursor_in_prompt(search: &SearchLine) -> String {
    let column = FIRST_COLUMN + width_of(SEARCH_PROMPT) + search.cursor;
    format!("\x1b[{PROMPT_ROW};{column}H")
}

fn painted(style: &str, text: &str) -> String {
    if text.is_empty() {
        String::new()
    } else {
        format!("{style}{text}{RESET}")
    }
}

fn faint_line(text: &str, width: usize) -> String {
    let text = fit(text, room(width, MARGIN.len()));
    format!("{FAINT}{MARGIN}{text}{RESET}")
}

#[cfg(test)]
mod tests;
