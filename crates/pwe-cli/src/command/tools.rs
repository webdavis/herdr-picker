use super::CommandError;
use crate::builtin;

pub(super) fn execute(words: &[&str]) -> Option<Result<(), CommandError>> {
    Some(match words {
        ["builtin", name, arguments @ ..] => builtin::print_asked(name, arguments),
        ["capture", arguments @ ..] => builtin::save_check(arguments),
        ["network", arguments @ ..] => builtin::network_tool(arguments),
        ["tabs", arguments @ ..] => builtin::tabs_tool(arguments),
        ["data", arguments @ ..] => builtin::data_command(arguments),
        ["git-preview", arguments @ ..] => {
            builtin::git_preview(arguments).map(|preview| print!("{preview}"))
        }
        _ => return None,
    })
}
