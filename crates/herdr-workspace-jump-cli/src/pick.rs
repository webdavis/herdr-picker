use std::env;
use std::io;
use std::path::Path;
use std::process;
use std::time::Duration;

use herdr_workspace_jump_adapters::{
    Layout, SocketWorkspaceDirectory, TerminalKeys, TerminalScreen, spawn_detached,
    wait_for_process_exit,
};
use herdr_workspace_jump_application::{JumpError, WorkspaceDirectory, pick};
use herdr_workspace_jump_domain::{
    Fuzzy, JumpTarget, Menu, ReadlineField, ReadlineKeymap, Workspace,
};

use crate::command::{CommandError, failed, spawned_jump_log_path};
use herdr_workspace_jump_protocol::SOCKET_PATH_VARIABLE;

const SPAWNER_EXIT_CAP: Duration = Duration::from_secs(2);

const LIVE_WORKSPACES_LOOKUP_CAP: Duration = Duration::from_secs(1);

pub(crate) const AFTER_PID: &str = "--after-pid";

const HERDR_DID_NOT_ANSWER: &str = "herdr did not answer, so open workspaces are not marked";

pub(crate) fn run(
    socket_path: Option<String>,
    active_id: Option<String>,
    targets: &[JumpTarget],
) -> Result<(), CommandError> {
    let layout = Layout::new(ReadlineKeymap::LEGEND, targets);
    let (live, layout) = match live_workspaces(socket_path.as_deref()) {
        Ok(live) => (live, layout),
        Err(_) => (Vec::new(), layout.noting(HERDR_DID_NOT_ANSWER)),
    };
    let menu = Menu::new(
        targets,
        &live,
        active_id.as_deref(),
        Fuzzy,
        ReadlineField::default(),
    );
    let mut screen = TerminalScreen::stdout(layout);
    match pick(menu, &ReadlineKeymap, &mut TerminalKeys, &mut screen).map_err(failed)? {
        Some(target) => spawn_jump(target, |binary, arguments| {
            spawn_detached(binary, arguments, Some(&spawned_jump_log_path()))
        }),
        None => Ok(()),
    }
}

fn live_workspaces(socket_path: Option<&str>) -> Result<Vec<Workspace>, JumpError> {
    let socket = socket_path
        .filter(|path| !path.is_empty())
        .ok_or_else(|| JumpError::Unreachable(format!("{SOCKET_PATH_VARIABLE} is unset")))?;
    SocketWorkspaceDirectory::at(Path::new(socket), LIVE_WORKSPACES_LOOKUP_CAP).list()
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
    use crate::socket_server::{Reply, SocketServer};

    const LIST: &str =
        r#"{"id":"req_1","result":{"workspaces":[{"workspace_id":"w11","label":"dotfiles"}]}}"#;

    #[test]
    fn the_live_workspaces_come_from_herdr_over_the_socket() {
        let server = SocketServer::start(vec![Reply::Line(LIST.to_string())]);

        let live = live_workspaces(Some(&server.path.to_string_lossy()));

        assert_eq!(
            live,
            Ok(vec![Workspace {
                workspace_id: "w11".to_string(),
                label: "dotfiles".to_string(),
            }])
        );
    }

    #[test]
    fn an_unset_or_dead_socket_is_reported_rather_than_read_as_nothing_open() {
        let unset = live_workspaces(None);
        let dead = live_workspaces(Some("/nonexistent/hwj.sock"));

        assert!(
            matches!(&unset, Err(JumpError::Unreachable(reason)) if reason.contains(SOCKET_PATH_VARIABLE)),
            "{unset:?}"
        );
        assert!(dead.is_err(), "{dead:?}");
    }

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
