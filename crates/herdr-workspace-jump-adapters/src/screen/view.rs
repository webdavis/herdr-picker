use herdr_workspace_jump_domain::{Frame, Row};

const BOLD_BLUE: &str = "\x1b[1;34m";
const FAINT: &str = "\x1b[2m";
const GREY_BAR: &str = "\x1b[48;5;236m";
const RESET: &str = "\x1b[0m";

const MARGIN: &str = " ";
const KEY_COLUMN_WIDTH: usize = 3;
const LABEL_INDENT: &str = "  ";
const KEYS_LEGEND: &str = "ctrl+n/p move  enter jump  esc close";

pub(super) fn render(frame: &Frame<'_>, width: usize) -> String {
    let mut lines: Vec<String> = frame
        .rows
        .iter()
        .map(|row| workspace_row(row, width))
        .collect();
    lines.push(String::new());
    lines.push(format!("{FAINT}{MARGIN}{KEYS_LEGEND}{RESET}"));
    lines.join("\n")
}

fn workspace_row(row: &Row<'_>, width: usize) -> String {
    let key = format!("{:>KEY_COLUMN_WIDTH$}", row.target.pick_key);
    let label = format!("{LABEL_INDENT}{}", row.target.label);
    let label_width = width.saturating_sub(MARGIN.len() + KEY_COLUMN_WIDTH);
    if row.selected {
        let rest = fill(&label, "", label_width);
        format!("{GREY_BAR}{MARGIN}{BOLD_BLUE}{key}{RESET}{GREY_BAR}{rest}{RESET}")
    } else {
        format!("{MARGIN}{BOLD_BLUE}{key}{RESET}{label}")
    }
}

fn fill(text: &str, trailing: &str, width: usize) -> String {
    let pad = width.saturating_sub(text.chars().count() + trailing.chars().count());
    format!("{text}{:pad$}{trailing}", "")
}

#[cfg(test)]
mod tests;
