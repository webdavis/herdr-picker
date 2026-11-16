use super::CommandError;
use super::daemon::socket;
use crate::notify;

pub(super) fn execute(words: &[&str]) -> Option<Result<(), CommandError>> {
    Some(match words {
        ["notify", options @ ..] => notify::run(&socket(), options),
        _ => return None,
    })
}
