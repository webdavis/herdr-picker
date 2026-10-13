use super::text::ELLIPSIS;
use super::*;
use crate::screen::fixtures::{browsing, layout, refused, searching, three_workspaces};

mod base;
mod fit;
mod layout;
mod leaders;
mod lit;
mod open;
mod refusal;
mod search;
mod stripes;
mod width;

const ON_BASE: &str = "\x1b[48;2;30;30;46m\x1b[22;38;2;205;214;244m";
const ON_BAR: &str = "\x1b[48;2;54;54;70m\x1b[22;38;2;205;214;244m";
const ON_STRIPE: &str = "\x1b[48;2;33;33;52m\x1b[22;38;2;205;214;244m";
const PLAIN: &str = "\x1b[22;38;2;205;214;244m";
const BOLD_BLUE: &str = "\x1b[1;38;2;137;180;250m";
const GREEN: &str = "\x1b[38;2;166;227;161m";
const GREY: &str = "\x1b[38;2;127;132;156m";
const DARK_GREY: &str = "\x1b[38;2;69;71;90m";
const RED: &str = "\x1b[38;2;243;139;168m";
const END: &str = "\x1b[0m";

fn legend_row(width: usize) -> String {
    let pad = " ".repeat(width.saturating_sub(" the keys".len()));
    format!("{ON_BASE}{GREY} the keys{PLAIN}{pad}{END}")
}

fn blank_row(width: usize) -> String {
    let pad = " ".repeat(width);
    format!("{ON_BASE}{pad}{END}")
}

fn visible_rows(drawn: &str) -> Vec<String> {
    drawn
        .lines()
        .map(without_escape_sequences)
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
        .collect()
}

fn without_escape_sequences(line: &str) -> String {
    let mut text = String::new();
    let mut rest = line;
    while let Some(start) = rest.find('\x1b') {
        text.push_str(&rest[..start]);
        let sequence = &rest[start..];
        rest = sequence
            .find(ends_an_escape_sequence)
            .map_or("", |end| &sequence[end + 1..]);
    }
    text.push_str(rest);
    text
}

fn ends_an_escape_sequence(character: char) -> bool {
    character.is_ascii_alphabetic()
}
