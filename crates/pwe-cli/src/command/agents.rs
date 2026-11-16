use super::CommandError;
use crate::agent_sessions;

pub(super) fn execute(words: &[&str]) -> Option<Result<(), CommandError>> {
    Some(match words {
        ["agents"] => agent_sessions::run(),
        _ => return None,
    })
}
