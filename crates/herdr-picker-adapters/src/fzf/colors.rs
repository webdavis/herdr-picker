use herdr_picker_domain::Theme;

use crate::screen::palette;

pub(super) fn colors(theme: Theme) -> String {
    let palette = palette(theme);
    let (base, bar, text) = (hex(palette.base), hex(palette.bar), hex(palette.text));
    let (faint, matched) = (hex(palette.faint), format!("{}:bold", hex(palette.matched)));
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
        ("pointer", &hex(palette.key)),
        ("gutter", &base),
        ("border", &hex(palette.leader)),
        ("preview-bg", &base),
        ("header", &faint),
        ("footer", &faint),
    ]
    .iter()
    .map(|(element, color)| format!("{element}:{color}"))
    .collect::<Vec<_>>()
    .join(",")
}

// The palette holds 24-bit SGR escapes ending in `r;g;bm`; fzf wants the same color as `#rrggbb`.
fn hex(sgr: &str) -> String {
    let mut channels: Vec<&str> = sgr.trim_end_matches('m').rsplit(';').take(3).collect();
    channels.reverse();
    let digits: String = channels
        .iter()
        .map(|channel| format!("{:02x}", channel.parse::<u8>().unwrap_or_default()))
        .collect();
    format!("#{digits}")
}
