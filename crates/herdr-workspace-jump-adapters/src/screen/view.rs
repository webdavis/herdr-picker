use herdr_workspace_jump_domain::{Frame, JumpTarget, Legend, Refusal, Row, SearchLine};

const BOLD_BLUE: &str = "\x1b[1;34m";
const FAINT: &str = "\x1b[2m";
const GREY_BAR: &str = "\x1b[48;5;236m";
const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";

const MARGIN: &str = " ";
const SEARCH_PROMPT: &str = " /";
const MIN_GAP_BEFORE_COUNT: usize = 2;
const KEY_COLUMN_WIDTH: usize = 3;
const LABEL_INDENT: &str = "  ";
const ALREADY_CURRENT: &str = "already in ";
const BEFORE_LABEL: usize = MARGIN.len() + KEY_COLUMN_WIDTH;
const PROMPT_ROW: usize = 1;
const FIRST_COLUMN: usize = 1;
const UNKNOWN_WIDTH: usize = 0;
const ELLIPSIS: char = '…';

pub(super) fn render(frame: &Frame<'_>, legend: Legend, width: usize, height: usize) -> String {
    let above = [prompt_row(frame, legend, width), String::new()];
    let below = [String::new(), bottom_row(frame, legend, width)];
    let room = height
        .checked_sub(above.len() + below.len())
        .unwrap_or(frame.rows.len());
    let mut workspaces = workspace_rows(frame, room, width);
    workspaces.resize(room, String::new());
    let mut drawn = [above.as_slice(), &workspaces, &below].concat().join("\n");
    if let Some(search) = &frame.search {
        drawn.push_str(&cursor_in_prompt(search));
    }
    drawn
}

pub(super) fn columns_to_fit(targets: &[JumpTarget], legend: Legend) -> usize {
    let rows = targets.iter().map(|target| row_width(&target.label));
    let refusals = targets
        .iter()
        .map(|target| line_width(&refusal_text(&target.label)));
    let faint_lines = [legend.search, legend.keys].into_iter().map(line_width);
    rows.chain(refusals)
        .chain(faint_lines)
        .max()
        .unwrap_or_default()
}

fn row_width(label: &str) -> usize {
    BEFORE_LABEL + LABEL_INDENT.len() + label.chars().count()
}

fn line_width(text: &str) -> usize {
    MARGIN.len() + text.chars().count()
}

fn refusal_text(label: &str) -> String {
    format!("{ALREADY_CURRENT}{label}")
}

fn bottom_row(frame: &Frame<'_>, legend: Legend, width: usize) -> String {
    match frame.refusal {
        Some(Refusal::AlreadyCurrent(target)) => {
            let text = fit(&refusal_text(&target.label), room(width, MARGIN.len()));
            format!("{RED}{MARGIN}{text}{RESET}")
        }
        None => faint_line(legend.keys, width),
    }
}

fn prompt_row(frame: &Frame<'_>, legend: Legend, width: usize) -> String {
    let Some(search) = &frame.search else {
        return faint_line(legend.search, width);
    };
    let typed = format!("{SEARCH_PROMPT}{}", search.text);
    let count = format!("{}/{}{MARGIN}", frame.rows.len(), frame.total);
    let gap = width
        .saturating_sub(typed.chars().count() + count.chars().count())
        .max(MIN_GAP_BEFORE_COUNT);
    format!("{typed}{:gap$}{FAINT}{count}{RESET}", "")
}

fn cursor_in_prompt(search: &SearchLine) -> String {
    let column = FIRST_COLUMN + SEARCH_PROMPT.chars().count() + search.cursor;
    format!("\x1b[{PROMPT_ROW};{column}H")
}

fn workspace_rows(frame: &Frame<'_>, room: usize, width: usize) -> Vec<String> {
    let selected = frame.rows.iter().position(|row| row.selected).unwrap_or(0);
    let top = (selected + 1).saturating_sub(room);
    let searching = frame.search.is_some();
    frame
        .rows
        .iter()
        .skip(top)
        .take(room)
        .map(|row| workspace_row(row, searching, width))
        .collect()
}

fn workspace_row(row: &Row<'_>, searching: bool, width: usize) -> String {
    let key = if searching {
        String::new()
    } else {
        row.target.pick_key.to_string()
    };
    let key = format!("{key:>KEY_COLUMN_WIDTH$}");
    let label = fit(
        &format!("{LABEL_INDENT}{}", row.target.label),
        room(width, BEFORE_LABEL),
    );
    if row.selected {
        let bar = fill(&label, width.saturating_sub(BEFORE_LABEL));
        format!("{GREY_BAR}{MARGIN}{BOLD_BLUE}{key}{RESET}{GREY_BAR}{bar}{RESET}")
    } else {
        format!("{MARGIN}{BOLD_BLUE}{key}{RESET}{label}")
    }
}

fn faint_line(text: &str, width: usize) -> String {
    let text = fit(text, room(width, MARGIN.len()));
    format!("{FAINT}{MARGIN}{text}{RESET}")
}

fn room(width: usize, used: usize) -> Option<usize> {
    (width != UNKNOWN_WIDTH).then(|| width.saturating_sub(used))
}

fn fit(text: &str, room: Option<usize>) -> String {
    let Some(room) = room.filter(|&room| text.chars().count() > room) else {
        return text.to_string();
    };
    let Some(kept) = room.checked_sub(1) else {
        return String::new();
    };
    text.chars().take(kept).chain([ELLIPSIS]).collect()
}

fn fill(text: &str, width: usize) -> String {
    let pad = width.saturating_sub(text.chars().count());
    format!("{text}{:pad$}", "")
}

#[cfg(test)]
mod tests;
