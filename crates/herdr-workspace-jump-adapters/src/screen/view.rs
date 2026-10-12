use herdr_workspace_jump_domain::{Frame, Row, SearchLine};

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
const SEARCH_HINT: &str = "/ search";
const KEYS_LEGEND: &str = "ctrl+n/p move  enter jump  esc close";

pub(super) fn render(frame: &Frame<'_>, width: usize) -> String {
    let above = [prompt_row(frame, width), String::new()];
    let below = [
        String::new(),
        format!("{FAINT}{MARGIN}{KEYS_LEGEND}{RESET}"),
    ];
    let searching = frame.search.is_some();
    let workspaces: Vec<String> = frame
        .rows
        .iter()
        .map(|row| workspace_row(row, searching, width))
        .collect();
    let mut drawn = [above.as_slice(), &workspaces, &below].concat().join("\n");
    if let Some(search) = &frame.search {
        drawn.push_str(&cursor_in_prompt(search));
    }
    drawn
}

fn prompt_row(frame: &Frame<'_>, width: usize) -> String {
    let Some(search) = &frame.search else {
        return format!("{FAINT}{MARGIN}{SEARCH_HINT}{RESET}");
    };
    let typed = format!("{SEARCH_PROMPT}{}", search.text);
    let count = format!("{}/{}{MARGIN}", frame.rows.len(), frame.total);
    let gap = width
        .saturating_sub(typed.chars().count() + count.chars().count())
        .max(MIN_GAP_BEFORE_COUNT);
    format!("{typed}{:gap$}{FAINT}{count}{RESET}", "")
}

fn cursor_in_prompt(search: &SearchLine) -> String {
    let column = FIRST_COLUMN + SEARCH_PROMPT.chars().count() + search.text.chars().count();
    format!("\x1b[{PROMPT_ROW};{column}H")
}

fn workspace_row(row: &Row<'_>, searching: bool, width: usize) -> String {
    let key = if searching {
        String::new()
    } else {
        row.target.pick_key.to_string()
    };
    let key = format!("{key:>KEY_COLUMN_WIDTH$}");
    let label = format!("{LABEL_INDENT}{}", row.target.label);
    let label_width = width.saturating_sub(MARGIN.len() + KEY_COLUMN_WIDTH);
    let rest = match (row.selected, row.current) {
        (false, false) => label,
        (_, true) => fill(&label, CURRENT_MARKER, label_width),
        (true, false) => fill(&label, "", label_width),
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

fn fill(text: &str, trailing: &str, width: usize) -> String {
    let pad = width.saturating_sub(text.chars().count() + trailing.chars().count());
    format!("{text}{:pad$}{trailing}", "")
}

#[cfg(test)]
mod tests;
