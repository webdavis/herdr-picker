use pwe_domain::Theme;

use crate::screen::palette;

pub(super) fn colors(theme: Theme) -> String {
    let palette = palette(theme);
    let (base, bar, text) = (
        sgr_to_hex(palette.base),
        sgr_to_hex(palette.bar),
        sgr_to_hex(palette.text),
    );
    let (faint, matched) = (
        sgr_to_hex(palette.faint),
        format!("{}:bold", sgr_to_hex(palette.matched)),
    );
    [
        ("bg", &base),
        ("bg+", &bar),
        ("fg", &text),
        ("fg+", &text),
        ("hl", &matched),
        ("hl+", &matched),
        ("info", &faint),
        ("prompt", &text),
        ("query", &text),
        ("pointer", &sgr_to_hex(palette.key)),
        ("gutter", &base),
        ("border", &sgr_to_hex(palette.leader)),
        ("preview-bg", &base),
        ("header", &faint),
        ("footer", &faint),
    ]
    .iter()
    .map(|(element, color)| format!("{element}:{color}"))
    .collect::<Vec<_>>()
    .join(",")
}

fn sgr_to_hex(sgr: &str) -> String {
    let mut channels: Vec<&str> = sgr.trim_end_matches('m').rsplit(';').take(3).collect();
    channels.reverse();
    let digits: String = channels
        .iter()
        .map(|channel| format!("{:02x}", channel.parse::<u8>().unwrap_or_default()))
        .collect();
    format!("#{digits}")
}
