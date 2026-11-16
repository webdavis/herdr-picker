use std::env;
use std::path::{Path, PathBuf};

use pwe_protocol::{PICKER_NAME_VARIABLE, PICKER_RESULT_VARIABLE, SOCKET_PATH_VARIABLE};

use super::{CommandError, config_path, picker_config_path};
use crate::{generate, handing, launch, pick, picker, picker_chrome, picker_popup};

pub(super) fn execute(words: &[&str]) -> Option<Result<(), CommandError>> {
    Some(match words {
        ["generate", options @ ..] => generate::run(options),
        ["launch", base] => launch::open(
            env::var(SOCKET_PATH_VARIABLE).ok(),
            env::var("HERDR_PLUGIN_ID").ok(),
            env::var("HERDR_PLUGIN_CONTEXT_JSON").ok(),
            base,
            &config_path(),
        ),
        ["launcher"] => launch::choose(),
        ["deliver", name, target, pick::AFTER_PID, spawner] => {
            handing::deliver(name, target, spawner)
        }
        ["run", name] => picker::run(env::var(SOCKET_PATH_VARIABLE).ok(), name),
        ["run-in-popup"] => picker_popup::answer_in_popup(
            env::var_os(PICKER_RESULT_VARIABLE).map(PathBuf::from),
            &env::var(PICKER_NAME_VARIABLE).unwrap_or_default(),
            &picker_config_path(),
        ),
        ["preview", cache, index] => {
            picker::preview(Path::new(cache), index).map(|preview| print!("{preview}"))
        }
        [
            part @ ("header" | "rows" | "live" | "view" | "footer" | "help"),
            cache,
            rest @ ..,
        ] => picker_chrome::answer(part, Path::new(cache), rest, &picker_config_path())
            .map(|answer| print!("{answer}")),
        ["init", shell] => picker::init(shell, &config_path()).map(|script| print!("{script}")),
        _ => return None,
    })
}
