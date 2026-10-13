use super::text::ELLIPSIS;
use super::*;
use crate::screen::fixtures::{browsing, layout, refused, searching, three_workspaces};

mod fit;
mod layout;
mod leaders;
mod lit;
mod open;
mod refusal;
mod search;
mod stripes;
mod width;

const LEGEND_ROW: &str = "\x1b[2m the keys\x1b[0m";

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
