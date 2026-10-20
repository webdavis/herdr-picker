use herdr_picker_domain::{Theme, View, header};

use crate::screen::palette;

mod chrome;

pub use chrome::{chooser_lines, error_footer, finder_actions, list_lines};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    Inline,
    Popup,
}

// How fzf calls back into herdr-picker about one run: both parts already shell-quoted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Helper<'a> {
    pub program: &'a str,
    pub cache: &'a str,
}

impl Helper<'_> {
    fn call(&self, subcommand: &str, rest: &str) -> String {
        format!("{} {subcommand} {}{rest}", self.program, self.cache)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chrome<'a> {
    pub helper: Helper<'a>,
    pub name: &'a str,
    pub description: &'a str,
}

const LAYOUT: [&str; 14] = [
    "--ansi",
    "--delimiter=\t",
    "--with-nth=2..",
    "--layout=reverse",
    "--info=inline-right",
    // Only the match count: the defaults' --multi and toggle-sort would add "(0)" and "+S".
    "--info-command=echo $FZF_MATCH_COUNT",
    "--prompt= /",
    "--pointer= ",
    "--highlight-line",
    "--no-separator",
    "--no-scrollbar",
    "--border=none",
    "--list-border=none",
    "--input-border=none",
];

const VIEW_KEYS: [(&str, &str); 3] = [
    ("ctrl-space", "choose"),
    ("enter", "enter {1}"),
    ("esc", "back"),
];

pub fn fzf_arguments(theme: Theme, chrome: &Chrome, placement: Placement) -> Vec<String> {
    let helper = chrome.helper;
    let mut arguments: Vec<String> = LAYOUT.iter().map(|argument| argument.to_string()).collect();
    arguments.push(format!("--preview={}", helper.call("preview", " {1}")));
    arguments.push("--preview-window=right,50%,border-left".to_string());
    arguments.push(format!(
        "--header={}",
        header(chrome.name, &View::Streamed, &[])
    ));
    if !chrome.description.is_empty() {
        arguments.push(format!("--footer={}", chrome.description));
    }
    arguments.push(format!(
        "--bind=load:transform-header({})+transform({})",
        helper.call("header", &format!(" {}", chrome.name)),
        helper.call("view", " load")
    ));
    for (key, event) in VIEW_KEYS {
        arguments.push(format!(
            "--bind={key}:transform({})",
            helper.call("view", &format!(" {event}"))
        ));
    }
    arguments.push(format!("--color={}", colors(theme)));
    if placement == Placement::Popup {
        arguments.push("--height=100%".to_string());
    }
    arguments
}

fn colors(theme: Theme) -> String {
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

#[cfg(test)]
mod tests;
