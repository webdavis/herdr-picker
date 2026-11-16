use ansi_to_tui::IntoText;
use pwe_domain::unstyled;
use ratatui::style::Style;
use ratatui::text::Text;

pub(super) fn ansi_text(text: &str) -> Text<'static> {
    text.as_bytes()
        .into_text()
        .unwrap_or_else(|_| Text::raw(unstyled(text)))
}

pub(super) fn style_of(sgr: &str) -> Style {
    ansi_text(&format!("{sgr} "))
        .lines
        .first()
        .and_then(|line| line.spans.first())
        .map(|span| span.style)
        .unwrap_or_default()
}
