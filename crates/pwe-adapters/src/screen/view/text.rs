const UNKNOWN_WIDTH: usize = 0;
const RESET: &str = "\x1b[0m";
pub(super) const ELLIPSIS: char = '…';

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Span {
    style: &'static str,
    text: String,
}

pub(super) fn span(style: &'static str, text: impl Into<String>) -> Span {
    Span {
        style,
        text: text.into(),
    }
}

pub(super) fn width_of(text: &str) -> usize {
    text.chars().count()
}

pub(super) fn width(spans: &[Span]) -> usize {
    spans.iter().map(|span| width_of(&span.text)).sum()
}

pub(super) fn room(width: usize, used: usize) -> Option<usize> {
    (width != UNKNOWN_WIDTH).then(|| width.saturating_sub(used))
}

pub(super) fn cut(spans: Vec<Span>, room: Option<usize>, ellipsis: &'static str) -> Vec<Span> {
    let Some(room) = room.filter(|&room| width(&spans) > room) else {
        return spans;
    };
    let Some(mut left) = room.checked_sub(1) else {
        return Vec::new();
    };
    let mut kept = Vec::new();
    for mut span in spans {
        span.text = span.text.chars().take(left).collect();
        left -= width_of(&span.text);
        kept.push(span);
        if left == 0 {
            break;
        }
    }
    kept.push(span(ellipsis, ELLIPSIS));
    kept
}

pub(super) fn line(spans: Vec<Span>, background: &str, plain: &str, width: usize) -> String {
    let runs: String = merged(&padded(spans, width))
        .iter()
        .map(|span| {
            if span.style.is_empty() {
                span.text.clone()
            } else {
                format!("{}{}{plain}", span.style, span.text)
            }
        })
        .collect();
    format!("{background}{plain}{runs}{RESET}")
}

fn padded(mut spans: Vec<Span>, width: usize) -> Vec<Span> {
    let missing = width.saturating_sub(self::width(&spans));
    spans.push(span("", " ".repeat(missing)));
    spans
}

fn merged(spans: &[Span]) -> Vec<Span> {
    let mut merged: Vec<Span> = Vec::new();
    for span in spans.iter().filter(|span| !span.text.is_empty()) {
        match merged.last_mut() {
            Some(last) if last.style == span.style => last.text.push_str(&span.text),
            _ => merged.push(span.clone()),
        }
    }
    merged
}
