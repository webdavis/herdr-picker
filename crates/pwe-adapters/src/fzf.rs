use pwe_domain::{Theme, bindable_keys};
use pwe_protocol::shell_word;

use colors::colors;

mod chrome;
mod colors;
mod ending;
mod step_finder;
mod unbound;

pub use chrome::{change_footer, chooser_lines, finder_actions, leave, list_lines};
pub(crate) use ending::fzf_ending;
pub use ending::{SAVE_QUICKFIX, SEND_ROWS, SWITCH_SOURCE};
pub use step_finder::FzfStepFinder;
pub use unbound::unbound_action;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    Inline,
    Popup,
}

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
    pub live: bool,
}

const MATCH_COUNT_WITHOUT_THE_MULTI_AND_SORT_MARKERS: &str = "--info-command=echo $FZF_MATCH_COUNT";

const LAYOUT: [&str; 16] = [
    "--ansi",
    "--delimiter=\t",
    "--with-nth=2..",
    "--layout=reverse",
    "--info=inline-right",
    MATCH_COUNT_WITHOUT_THE_MULTI_AND_SORT_MARKERS,
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

const VIEW_KEYS: [(&str, &str); 4] = [
    ("ctrl-space", "choose"),
    ("esc", "back"),
    ("ctrl-q", "quickfix {+1}"),
    ("alt-o", "send"),
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
            "--bind=change:reload-sync({})",
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
    let row_keys = bindable_keys().map(|key| {
        let event = format!("key {} {{+1}}", shell_word(key.name()));
        (key.name().to_string(), event)
    });
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
