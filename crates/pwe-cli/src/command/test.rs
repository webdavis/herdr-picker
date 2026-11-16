mod event_printer;
mod flags;
mod output;
mod run;
mod run_runner;
mod run_target;
mod runners;
mod system_clock;

use super::CommandError;

pub(super) fn execute(words: &[&str]) -> Option<Result<(), CommandError>> {
    let ["test", rest @ ..] = words else {
        return None;
    };
    Some(match rest {
        ["runners", flags @ ..] => runners::runners(flags),
        ["run", flags @ ..] => run::run(flags),
        _ => Err(CommandError::Usage),
    })
}
