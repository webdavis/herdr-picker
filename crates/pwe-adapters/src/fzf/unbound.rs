// What fzf does with a key nothing binds; binding every key a row may claim would otherwise take
// these from typing and moving on rows that do not claim them.
const UNBOUND: [(&str, &str); 15] = [
    ("ctrl-a", "beginning-of-line"),
    ("ctrl-b", "backward-char"),
    ("ctrl-d", "delete-char/eof"),
    ("ctrl-e", "end-of-line"),
    ("ctrl-f", "forward-char"),
    ("ctrl-g", "abort"),
    ("ctrl-j", "down"),
    ("ctrl-k", "up"),
    ("ctrl-l", "clear-screen"),
    ("ctrl-u", "unix-line-discard"),
    ("ctrl-w", "unix-word-rubout"),
    ("ctrl-y", "yank"),
    ("alt-b", "backward-word"),
    ("alt-d", "kill-word"),
    ("alt-f", "forward-word"),
];

pub fn unbound_action(key: &str) -> Option<&'static str> {
    UNBOUND
        .iter()
        .find(|(name, _)| *name == key)
        .map(|(_, action)| *action)
}
