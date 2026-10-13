pub(super) const UNKNOWN_WIDTH: usize = 0;
pub(super) const ELLIPSIS: char = '…';

pub(super) fn room(width: usize, used: usize) -> Option<usize> {
    (width != UNKNOWN_WIDTH).then(|| width.saturating_sub(used))
}

pub(super) fn fit(text: &str, room: Option<usize>) -> String {
    let Some(room) = room.filter(|&room| text.chars().count() > room) else {
        return text.to_string();
    };
    let Some(kept) = room.checked_sub(1) else {
        return String::new();
    };
    text.chars().take(kept).chain([ELLIPSIS]).collect()
}

pub(super) fn fill(text: &str, width: usize) -> String {
    let pad = width.saturating_sub(text.chars().count());
    format!("{text}{:pad$}", "")
}

pub(super) fn width_of(text: &str) -> usize {
    text.chars().count()
}
