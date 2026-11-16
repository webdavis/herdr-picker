use super::{CommandError, config_path};
use crate::rex;

pub(super) fn execute(words: &[&str]) -> Option<Result<(), CommandError>> {
    Some(match words {
        ["rex", "key", base] => rex::key(base),
        ["launcher", base, rex::UNDER, block] => rex::launcher(base, block),
        ["init", "rex"] => rex::lua(&config_path()).map(|script| print!("{script}")),
        _ => return None,
    })
}
