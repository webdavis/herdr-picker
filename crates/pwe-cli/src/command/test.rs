mod flags;
mod output;
mod runners;

use super::CommandError;

pub(super) fn execute(words: &[&str]) -> Option<Result<(), CommandError>> {
    let ["test", rest @ ..] = words else {
        return None;
    };
    Some(match rest {
        ["runners", flags @ ..] => runners::runners(flags),
        _ => Err(CommandError::Usage),
    })
}
