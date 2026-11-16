use super::CommandError;
use crate::inbox;

pub(super) fn execute(words: &[&str]) -> Option<Result<(), CommandError>> {
    Some(match words {
        ["inbox", options @ ..] => inbox::run(options),
        _ => return None,
    })
}
