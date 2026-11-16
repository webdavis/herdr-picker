mod event_printer;
mod flags;
mod history;
mod list;
mod output;
mod results;
mod run;
mod run_again;
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
        ["list", flags @ ..] => list::list(flags),
        ["run", flags @ ..] => run::run(flags),
        ["results", flags @ ..] => results::results(flags),
        _ => Err(CommandError::Usage),
    })
}
