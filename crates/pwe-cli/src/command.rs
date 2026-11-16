use std::env;
use std::path::{Path, PathBuf};
use std::time::Duration;

use pwe_adapters::{
    FileWorkspaceHistory, SocketWorkspaceDirectory, config_file, plugin_state_dir, read_config,
};
use pwe_protocol::{
    ACTIVE_WORKSPACE_VARIABLE, PICKER_CONFIG_VARIABLE, PICKER_NAME_VARIABLE,
    PICKER_RESULT_VARIABLE, SOCKET_PATH_VARIABLE,
};

use crate::{
    agent_hook, builtin, generate, handing, last_workspace, launch, path::expand_tilde, pick,
    picker, picker_chrome, picker_popup, popup, record, run,
};

pub(crate) const USAGE: &str = "usage: pwe jump <label> <cwd> [--after-pid <pid>] | pwe pick | pwe popup | pwe last-workspace | pwe record | pwe agent-changed | pwe launch <base> | pwe launcher | pwe deliver <picker> <pane> --after-pid <pid> | pwe generate --output <directory> [--config <file>] | pwe run <picker> | pwe run-in-popup | pwe preview <rows-file> <index> | pwe header <rows-file> | pwe rows <rows-file> | pwe live <rows-file> <query> | pwe view <rows-file> load|choose|back|key <key> <index>... | pwe footer <rows-file> [index] | pwe help <rows-file> [index] | pwe builtin <name> [args] | pwe git-preview <diff|stages|blob> <args> | pwe capture <output> <cwd> <category> <framework> -- <command>... | pwe network ping <address> <page> | pwe network wake <mac> <broadcast>... | pwe tabs focus <browser> <window> <tab> [space] | pwe init bash";

#[derive(Debug)]
pub(crate) enum CommandError {
    Usage,
    Failed(String),
}

pub(crate) fn failed(refusal: impl std::fmt::Display) -> CommandError {
    CommandError::Failed(refusal.to_string())
}

pub(crate) fn config_path() -> PathBuf {
    config_file(
        env::var("HERDR_PLUGIN_CONFIG_DIR").ok().as_deref(),
        env::var("XDG_CONFIG_HOME").ok().as_deref(),
        env::var("HOME").ok().as_deref(),
    )
}

pub(crate) fn picker_config_path() -> PathBuf {
    env::var_os(PICKER_CONFIG_VARIABLE).map_or_else(config_path, PathBuf::from)
}

pub(crate) fn state_dir() -> PathBuf {
    plugin_state_dir(
        env::var("HERDR_PLUGIN_STATE_DIR").ok().as_deref(),
        env::var("XDG_STATE_HOME").ok().as_deref(),
        env::var("HOME").ok().as_deref(),
    )
}

fn jump_to(label: &str, raw_cwd: &str) -> Result<(), CommandError> {
    let cwd =
        expand_tilde(raw_cwd, env::var("HOME").ok().as_deref()).map_err(CommandError::Failed)?;
    run::run(
        env::var(SOCKET_PATH_VARIABLE).ok(),
        env::var("HERDR_BIN_PATH").ok(),
        label,
        &cwd.to_string_lossy(),
    )
    .map(|_| ())
    .map_err(|failure| CommandError::Failed(format!("could not jump to {label}: {failure}")))
}

const PICK_LOOKUP_CAP: Duration = Duration::from_secs(1);

pub(crate) fn execute(arguments: &[String]) -> Result<(), CommandError> {
    let words: Vec<&str> = arguments.iter().map(String::as_str).collect();
    match words.as_slice() {
        ["jump", label, raw_cwd] => jump_to(label, raw_cwd),
        ["jump", label, raw_cwd, pick::AFTER_PID, spawner] => {
            pick::wait_for_spawner(spawner.parse().map_err(|_| CommandError::Usage)?);
            jump_to(label, raw_cwd)
        }
        ["pick"] => pick::run(
            &mut SocketWorkspaceDirectory::at(
                Path::new(&env::var(SOCKET_PATH_VARIABLE).unwrap_or_default()),
                PICK_LOOKUP_CAP,
            ),
            env::var(ACTIVE_WORKSPACE_VARIABLE).ok(),
            &read_config(&config_path()).map_err(failed)?,
        ),
        ["popup"] => popup::run(
            env::var(SOCKET_PATH_VARIABLE).ok(),
            env::var("HERDR_PLUGIN_ID").ok(),
            env::var("HERDR_WORKSPACE_ID").ok(),
            &read_config(&config_path()).map_err(failed)?.targets,
        ),
        ["last-workspace"] => last_workspace::run(
            env::var(SOCKET_PATH_VARIABLE).ok(),
            env::var("HERDR_BIN_PATH").ok(),
            &mut FileWorkspaceHistory::at(&record::mru_path()),
        )
        .map(|_| ())
        .map_err(|failure| {
            CommandError::Failed(format!(
                "could not toggle to the previous workspace: {failure}"
            ))
        }),
        ["generate", options @ ..] => generate::run(options),
        ["record"] => {
            record::run(&env::var("HERDR_PLUGIN_EVENT_JSON").unwrap_or_default());
            Ok(())
        }
        ["agent-changed"] => {
            agent_hook::run(&env::var("HERDR_PLUGIN_EVENT_JSON").unwrap_or_default());
            Ok(())
        }
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
        ["builtin", name, arguments @ ..] => builtin::print_asked(name, arguments),
        ["capture", arguments @ ..] => builtin::save_check(arguments),
        ["network", arguments @ ..] => builtin::network_tool(arguments),
        ["tabs", arguments @ ..] => builtin::tabs_tool(arguments),
        ["data", arguments @ ..] => builtin::data_command(arguments),
        ["git-preview", arguments @ ..] => {
            builtin::git_preview(arguments).map(|preview| print!("{preview}"))
        }
        ["inbox", options @ ..] => crate::inbox::run(options),
        ["init", shell] => picker::init(shell, &config_path()).map(|script| print!("{script}")),
        _ => Err(CommandError::Usage),
    }
}

#[cfg(test)]
mod tests;
