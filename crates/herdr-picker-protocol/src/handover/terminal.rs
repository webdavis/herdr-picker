use std::path::Path;

use crate::shell::split_words;

const INTERACTIVE: [&str; 15] = [
    "nvim", "vim", "vi", "hx", "nano", "emacs", "micro", "ssh", "mosh", "sudo", "less", "man",
    "top", "htop", "btop",
];
const SHELL_INTO: [&str; 3] = ["docker", "podman", "kubectl"];

// An agent's `!` runs a command without a terminal, so an editor, a login or a password prompt
// would hang there.
pub(super) fn needs_terminal(command: &str, editor: &str) -> bool {
    let editor = split_words(editor).unwrap_or_default();
    let editor = editor.first().map(|word| program_name(word));
    let words = split_words(command).unwrap_or_default();
    let mut programs = words
        .iter()
        .skip_while(|word| word.contains('='))
        .map(|word| program_name(word));
    match (programs.next(), programs.next()) {
        (Some(program), next) => {
            INTERACTIVE.contains(&program)
                || Some(program) == editor
                || (SHELL_INTO.contains(&program) && next == Some("exec"))
        }
        (None, _) => false,
    }
}

fn program_name(word: &str) -> &str {
    Path::new(word)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(word)
}
