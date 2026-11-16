use pwe_domain::finder::ListLine;
use pwe_domain::{PickerRow, Theme, View, unstyled};
use ratatui::style::Style;
use ratatui::text::{Line, Span};

use super::styled::{ansi_text, style_of};
use crate::picker_text::listed_labels;
use crate::screen::palette;

const MARK: &str = "┃";

pub(super) fn labels(lines: &[ListLine]) -> Vec<String> {
    let rows: Vec<PickerRow> = lines.iter().filter_map(|line| line.row.cloned()).collect();
    if rows.len() < lines.len() {
        return lines.iter().map(|line| line.label.to_string()).collect();
    }
    listed_labels(&rows, &View::Streamed { group: None })
        .into_iter()
        .map(|(_, label)| label)
        .collect()
}

pub(super) fn list_line(line: &ListLine, label: &str, theme: Theme) -> Line<'static> {
    let palette = palette(theme);
    let matched = style_of(palette.matched);
    let column = unstyled(label)
        .chars()
        .count()
        .saturating_sub(unstyled(line.label).chars().count());
    let gutter = match line.marked {
        true => Span::styled(format!(" {MARK}"), style_of(palette.key)),
        false => Span::raw("  "),
    };
    let characters = ansi_text(label)
        .lines
        .into_iter()
        .next()
        .unwrap_or_default()
        .spans
        .into_iter()
        .flat_map(|span| {
            let style = span.style;
            span.content
                .chars()
                .map(move |character| (character, style))
                .collect::<Vec<_>>()
        })
        .enumerate()
        .map(|(place, (character, style))| {
            let found = place >= column && line.matched.contains(&(place - column));
            let style = if found { style.patch(matched) } else { style };
            Span::styled(character.to_string(), style)
        });
    let style = match line.current {
        true => style_of(palette.bar),
        false => Style::default(),
    };
    Line::from_iter(std::iter::once(gutter).chain(characters)).style(style)
}
