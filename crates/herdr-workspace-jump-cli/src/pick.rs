use std::env;
use std::io;
use std::path::Path;
use std::process;
use std::time::Duration;

use herdr_workspace_jump_adapters::{
    SocketWorkspaceDirectory, TerminalKeys, TerminalScreen, read_jump_targets, spawn_detached,
    wait_for_process_exit,
};
use herdr_workspace_jump_application::{WorkspaceDirectory, pick};
use herdr_workspace_jump_domain::{Fuzzy, JumpTarget, Menu, ReadlineField, ReadlineKeymap};
use herdr_workspace_jump_protocol::ACTIVE_WORKSPACE_VARIABLE;

use crate::command::{CommandError, config_path, failed, spawned_jump_log_path};

const SPAWNER_EXIT_CAP: Duration = Duration::from_secs(2);

const CURRENT_MARKER_LOOKUP_CAP: Duration = Duration::from_secs(1);

pub(crate) const AFTER_PID: &str = "--after-pid";

pub(crate) fn run() -> Result<(), CommandError> {
    let targets = read_jump_targets(&config_path()).map_err(failed)?;
    let current = current_label();
    let menu = Menu::new(
        &targets,
        current.as_deref(),
        Fuzzy,
        ReadlineField::default(),
    );
    let mut screen = TerminalScreen::stdout(ReadlineKeymap::LEGEND);
    match pick(menu, &ReadlineKeymap, &mut TerminalKeys, &mut screen).map_err(failed)? {
        Some(target) => spawn_jump(target, |binary, arguments| {
            spawn_detached(binary, arguments, Some(&spawned_jump_log_path()))
        }),
        None => Ok(()),
    }
}

fn current_label() -> Option<String> {
    let active = env::var(ACTIVE_WORKSPACE_VARIABLE).ok()?;
    let socket = env::var("HERDR_SOCKET_PATH").ok()?;
    SocketWorkspaceDirectory::at(Path::new(&socket), CURRENT_MARKER_LOOKUP_CAP)
        .list()
        .ok()?
        .into_iter()
        .find(|workspace| workspace.workspace_id == active)
        .map(|workspace| workspace.label)
}

fn spawn_jump(
    target: &JumpTarget,
    spawn: impl FnOnce(&Path, &[&str]) -> io::Result<()>,
) -> Result<(), CommandError> {
    let binary = env::current_exe()
        .map_err(|failure| failed(format!("cannot find this binary: {failure}")))?;
    let popup = process::id().to_string();
    spawn(&binary, &jump_arguments(target, &popup))
        .map_err(|failure| failed(format!("cannot start the jump: {failure}")))
}

fn jump_arguments<'a>(target: &'a JumpTarget, popup: &'a str) -> [&'a str; 5] {
    ["jump", &target.label, &target.directory, AFTER_PID, popup]
}

pub(crate) fn wait_for_spawner(pid: u32) {
    wait_for_process_exit(pid, SPAWNER_EXIT_CAP);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target() -> JumpTarget {
        JumpTarget {
            action_id: "jump_ivy".to_string(),
            label: "Ivy".to_string(),
            directory: "~/workspaces/Ivy".to_string(),
            pick_key: 'i',
        }
    }

    #[test]
    fn the_spawned_jump_is_this_binary_with_the_label_the_directory_and_this_pid() {
        let mut spawned = Vec::new();
        let this_binary = env::current_exe().expect("a running binary has a path");

        let started = spawn_jump(&target(), |binary, arguments| {
            assert_eq!(binary, this_binary, "the popup spawned something else");
            spawned = arguments.iter().map(|word| word.to_string()).collect();
            Ok(())
        });

        assert!(started.is_ok(), "{started:?}");
        assert_eq!(
            spawned,
            vec![
                "jump",
                "Ivy",
                "~/workspaces/Ivy",
                "--after-pid",
                &process::id().to_string()
            ]
        );
    }

    #[test]
    fn a_spawn_that_fails_is_reported_rather_than_swallowed() {
        let refused = spawn_jump(&target(), |_, _| Err(io::Error::other("no such binary")));

        assert!(
            matches!(&refused, Err(CommandError::Failed(reason)) if reason.contains("no such binary")),
            "{refused:?}"
        );
    }
}
