use herdr_workspace_jump_domain::{Frame, Row};

const BOLD_BLUE: &str = "\x1b[1;34m";
const FAINT: &str = "\x1b[2m";
const GREY_BAR: &str = "\x1b[48;5;236m";
const RESET: &str = "\x1b[0m";
const KEYS_LEGEND: &str = "ctrl+n/p move  enter jump  esc close";

pub(super) fn render(frame: &Frame<'_>, width: usize) -> String {
    let mut lines: Vec<String> = frame
        .rows
        .iter()
        .map(|row| workspace_row(row, width))
        .collect();
    lines.push(String::new());
    lines.push(format!("{FAINT} {KEYS_LEGEND}{RESET}"));
    lines.join("\n")
}

fn workspace_row(row: &Row<'_>, width: usize) -> String {
    let key = format!("{:>3}", row.target.pick_key);
    let label = format!("  {}", row.target.label);
    if row.selected {
        let bar = fill(&label, "", width.saturating_sub(4));
        format!("{GREY_BAR} {BOLD_BLUE}{key}{RESET}{GREY_BAR}{bar}{RESET}")
    } else {
        format!(" {BOLD_BLUE}{key}{RESET}{label}")
    }
}

fn fill(text: &str, trailing: &str, width: usize) -> String {
    let pad = width.saturating_sub(text.chars().count() + trailing.chars().count());
    format!("{text}{:pad$}{trailing}", "")
}

#[cfg(test)]
mod tests;
