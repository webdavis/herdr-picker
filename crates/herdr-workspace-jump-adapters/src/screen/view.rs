use herdr_workspace_jump_domain::{Frame, Legend, Row, SearchLine};

const BOLD_BLUE: &str = "\x1b[1;34m";
const FAINT: &str = "\x1b[2m";
const GREY_BAR: &str = "\x1b[48;5;236m";
const RESET: &str = "\x1b[0m";

const MARGIN: &str = " ";
const SEARCH_PROMPT: &str = " /";
const MIN_GAP_BEFORE_COUNT: usize = 2;
const KEY_COLUMN_WIDTH: usize = 3;
const LABEL_INDENT: &str = "  ";
const CURRENT_MARKER: &str = "current ";
const PROMPT_ROW: usize = 1;
const FIRST_COLUMN: usize = 1;
const UNKNOWN_WIDTH: usize = 0;
const ELLIPSIS: char = '…';

pub(super) fn render(frame: &Frame<'_>, legend: Legend, width: usize, height: usize) -> String {
    let above = [prompt_row(frame, legend, width), String::new()];
    let below = [String::new(), faint_line(legend.keys, width)];
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
    let before_label = MARGIN.len() + KEY_COLUMN_WIDTH;
    let label = fit(
        &format!("{LABEL_INDENT}{}", row.target.label),
        room(width, before_label),
    );
    let marked = row.current
        && room(width, before_label + label.chars().count())
            .is_none_or(|left| left >= CURRENT_MARKER.chars().count());
    let marker = if marked { CURRENT_MARKER } else { "" };
    let rest = match (row.selected, row.current) {
        (false, false) => label,
        _ => fill(&label, marker, width.saturating_sub(before_label)),
    };
    match (row.selected, row.current) {
        (false, false) => format!("{MARGIN}{BOLD_BLUE}{key}{RESET}{rest}"),
        (false, true) => format!("{FAINT}{MARGIN}{key}{rest}{RESET}"),
        (true, false) => {
            format!("{GREY_BAR}{MARGIN}{BOLD_BLUE}{key}{RESET}{GREY_BAR}{rest}{RESET}")
        }
        (true, true) => format!("{GREY_BAR}{FAINT}{MARGIN}{key}{rest}{RESET}"),
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

fn fill(text: &str, trailing: &str, width: usize) -> String {
    let pad = width.saturating_sub(text.chars().count() + trailing.chars().count());
    format!("{text}{:pad$}{trailing}", "")
}

#[cfg(test)]
mod tests;
