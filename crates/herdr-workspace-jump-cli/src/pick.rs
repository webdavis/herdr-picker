use std::env;
use std::io;
use std::path::Path;
use std::process;
use std::time::Duration;

use herdr_workspace_jump_adapters::{
    Layout, PluginConfig, TerminalKeys, TerminalScreen, spawn_detached, wait_for_process_exit,
};
use herdr_workspace_jump_application::{WorkspaceDirectory, pick};
use herdr_workspace_jump_domain::{
    Fuzzy, JumpTarget, Menu, ReadlineField, ReadlineKeymap, Workspace,
};

use crate::command::{CommandError, failed, spawned_jump_log_path};

const SPAWNER_EXIT_CAP: Duration = Duration::from_secs(2);

pub(crate) const AFTER_PID: &str = "--after-pid";

const HERDR_DID_NOT_ANSWER: &str = "herdr did not answer, so open workspaces are not marked";

pub(crate) fn run(
    herdr: &mut impl WorkspaceDirectory,
    active_id: Option<String>,
    config: &PluginConfig,
) -> Result<(), CommandError> {
    let layout = Layout::new(ReadlineKeymap::LEGEND, &config.targets).look(config.look);
    let (live, layout) = open_workspaces(herdr, layout);
    let menu = Menu::new(
        &config.targets,
        &live,
        active_id.as_deref(),
        Fuzzy,
        ReadlineField::default(),
    );
    let mut screen = TerminalScreen::stdout(layout.hiding(menu.current_label()));
    match pick(menu, &ReadlineKeymap, &mut TerminalKeys, &mut screen).map_err(failed)? {
        Some(target) => spawn_jump(target, |binary, arguments| {
            spawn_detached(binary, arguments, Some(&spawned_jump_log_path()))
        }),
        None => Ok(()),
    }
}

fn open_workspaces(
    herdr: &mut impl WorkspaceDirectory,
    layout: Layout,
) -> (Vec<Workspace>, Layout) {
    match herdr.list() {
        Ok(live) => (live, layout),
        Err(_) => (Vec::new(), layout.noting(HERDR_DID_NOT_ANSWER)),
    }
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
mod tests;
