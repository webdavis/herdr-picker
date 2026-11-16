use super::CommandError;
use crate::rex;

pub(super) fn execute(words: &[&str]) -> Option<Result<(), CommandError>> {
    Some(match words {
        ["rex", "key", base] => rex::key(base),
        _ => return None,
    })
}
