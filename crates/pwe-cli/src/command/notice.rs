use super::CommandError;
use crate::notice;

pub(super) fn execute(words: &[&str]) -> Option<Result<(), CommandError>> {
    Some(match words {
        ["notice", "--level", level, "--until", until] => notice::run(level, until),
        _ => return None,
    })
}
