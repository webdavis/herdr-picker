// The colors the fzf-pickers suite drew git rows in.
const GREEN: &str = "\x1b[38;2;166;227;161m";
const YELLOW: &str = "\x1b[38;2;249;226;175m";
const BLUE: &str = "\x1b[38;2;137;180;250m";
const MAUVE: &str = "\x1b[38;2;203;166;247m";
const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

pub(super) fn green(text: &str) -> String {
    format!("{GREEN}{text}{RESET}")
}

pub(super) fn yellow(text: &str) -> String {
    format!("{YELLOW}{text}{RESET}")
}

pub(super) fn blue(text: &str) -> String {
    format!("{BLUE}{text}{RESET}")
}

pub(super) fn mauve(text: &str) -> String {
    format!("{MAUVE}{text}{RESET}")
}

pub(super) fn dim(text: &str) -> String {
    format!("{DIM}{text}{RESET}")
}

pub(super) fn bold(text: &str) -> String {
    format!("{BOLD}{text}{RESET}")
}

const RED: &str = "\x1b[38;2;243;139;168m";
const TEAL: &str = "\x1b[38;2;148;226;213m";

pub(super) fn red(text: &str) -> String {
    format!("{RED}{text}{RESET}")
}

pub(super) fn teal(text: &str) -> String {
    format!("{TEAL}{text}{RESET}")
}
