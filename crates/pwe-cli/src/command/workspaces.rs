use std::env;
use std::path::Path;
use std::time::Duration;

use pwe_adapters::{FileWorkspaceHistory, SocketWorkspaceDirectory, read_config};
use pwe_protocol::{ACTIVE_WORKSPACE_VARIABLE, SOCKET_PATH_VARIABLE};

use super::{CommandError, config_path, failed};
use crate::{agent_hook, last_workspace, path::expand_tilde, pick, popup, record, run};

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

pub(super) fn execute(words: &[&str]) -> Option<Result<(), CommandError>> {
    Some(match words {
        ["jump", label, raw_cwd] => jump_to(label, raw_cwd),
        ["jump", label, raw_cwd, pick::AFTER_PID, spawner] => match spawner.parse() {
            Ok(spawner) => {
                pick::wait_for_spawner(spawner);
                jump_to(label, raw_cwd)
            }
            Err(_) => Err(CommandError::Usage),
        },
        ["pick"] => read_config(&config_path())
            .map_err(failed)
            .and_then(|config| {
                pick::run(
                    &mut SocketWorkspaceDirectory::at(
                        Path::new(&env::var(SOCKET_PATH_VARIABLE).unwrap_or_default()),
                        PICK_LOOKUP_CAP,
                    ),
                    env::var(ACTIVE_WORKSPACE_VARIABLE).ok(),
                    &config,
                )
            }),
        ["popup"] => read_config(&config_path())
            .map_err(failed)
            .and_then(|config| {
                popup::run(
                    env::var(SOCKET_PATH_VARIABLE).ok(),
                    env::var("HERDR_PLUGIN_ID").ok(),
                    env::var("HERDR_WORKSPACE_ID").ok(),
                    &config.targets,
                )
            }),
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
        ["record"] => {
            record::run(&env::var("HERDR_PLUGIN_EVENT_JSON").unwrap_or_default());
            Ok(())
        }
        ["agent-changed"] => {
            agent_hook::run(&env::var("HERDR_PLUGIN_EVENT_JSON").unwrap_or_default());
            Ok(())
        }
        _ => return None,
    })
}
