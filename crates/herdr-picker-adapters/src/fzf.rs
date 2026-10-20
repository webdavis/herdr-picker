use herdr_picker_domain::{Theme, bindable_keys};

use colors::colors;

mod chrome;
mod colors;
mod unbound;

pub use chrome::{
    SAVE_QUICKFIX, SWITCH_SOURCE, change_footer, chooser_lines, error_footer, finder_actions,
    help_page, leave, list_lines, step_footer,
};
pub use unbound::unbound_action;

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
    pub header: &'a str,
    pub description: &'a str,
    pub query: &'a str,
    // The command matches what is typed, so fzf only shows its rows and asks again on each change.
    pub live: bool,
}

const LAYOUT: [&str; 16] = [
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
    "--multi",
    "--print-query",
];

const VIEW_KEYS: [(&str, &str); 3] = [
    ("ctrl-space", "choose"),
    ("esc", "back"),
    ("ctrl-q", "quickfix {+1}"),
];

pub fn fzf_arguments(theme: Theme, chrome: &Chrome, placement: Placement) -> Vec<String> {
    let helper = chrome.helper;
    let mut arguments: Vec<String> = LAYOUT.iter().map(|argument| argument.to_string()).collect();
    arguments.push(format!("--preview={}", helper.call("preview", " {1}")));
    arguments.push("--preview-window=right,50%,border-left".to_string());
    arguments.push(format!("--header={}", chrome.header));
    arguments.push(format!("--query={}", chrome.query));
    if chrome.live {
        arguments.push("--disabled".to_string());
        arguments.push(format!(
            "--bind=change:reload({})",
            helper.call("live", " {q}")
        ));
    }
    if !chrome.description.is_empty() {
        arguments.push(format!("--footer={}", chrome.description));
    }
    arguments.push(format!(
        "--bind=load:transform-header({})+transform({})",
        helper.call("header", ""),
        helper.call("view", " load")
    ));
    arguments.push(format!(
        "--bind=focus:transform-footer({})",
        helper.call("footer", " {1}")
    ));
    arguments.push(format!(
        "--bind=f1:preview({})",
        helper.call("help", " {1}")
    ));
    let row_keys =
        bindable_keys().map(|key| (key.name().to_string(), format!("key {} {{+1}}", key.name())));
    let view_keys = VIEW_KEYS.map(|(key, event)| (key.to_string(), event.to_string()));
    for (key, event) in view_keys.into_iter().chain(row_keys) {
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

#[cfg(test)]
mod tests;
