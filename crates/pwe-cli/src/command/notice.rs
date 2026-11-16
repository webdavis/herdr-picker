use super::CommandError;
use super::daemon::socket;
use crate::{notice, notice_keys};

pub(super) fn execute(words: &[&str]) -> Option<Result<(), CommandError>> {
    Some(match words {
        ["notice", "--level", level, "--until", until] => notice::run(level, until),
        ["notice", key_words @ ..] => notice_keys::run(&socket(), key_words),
        _ => return None,
    })
}
