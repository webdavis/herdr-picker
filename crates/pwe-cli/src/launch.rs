use std::path::Path;
use std::{env, process};

use pwe_adapters::{
    DEADLINE, Layout, SocketPopupHost, TerminalInput, TerminalScreen, read_picker_config,
    spawn_detached, spawned_jump_log_file,
};
use pwe_application::{open_picker, pick};
use pwe_domain::{Fuzzy, LauncherKeymap, Menu, ReadlineField, launcher_base, launcher_targets};
use pwe_protocol::{PICKER_BASE_VARIABLE, PICKER_TARGET_VARIABLE, PopupPane, parse_context_pane};

use crate::command::{CommandError, config_path, failed, state_dir};
use crate::pick::AFTER_PID;
use crate::picker;
use crate::popup::present;

pub(crate) fn open(
    socket_path: Option<String>,
    plugin_id: Option<String>,
    context: Option<String>,
    base: &str,
    config: &Path,
) -> Result<(), CommandError> {
    let stroke = launcher_base(base).ok_or(CommandError::Usage)?;
    let (target, cwd) = context
        .as_deref()
        .and_then(parse_context_pane)
        .ok_or_else(|| {
            failed("herdr named no focused pane, so a result would have nowhere to go")
        })?;
    let socket_path = present(socket_path, "HERDR_SOCKET_PATH")?;
    let plugin_id = present(plugin_id, "HERDR_PLUGIN_ID")?;
    let offered = launcher_targets(&read_picker_config(config).map_err(failed)?.pickers, stroke);
    let mut host = SocketPopupHost::at(
        Path::new(&socket_path),
        DEADLINE,
        &plugin_id,
        PopupPane::launcher(base, &target, cwd.as_deref()),
    );
    open_picker(
        &mut host,
        Layout::new(LauncherKeymap::LEGEND, &offered).columns_to_fit(),
    )
    .map_err(|failure| failed(format!("could not open the launcher: {failure}")))
}

pub(crate) fn choose() -> Result<(), CommandError> {
    let variable = |name| {
        env::var(name)
            .ok()
            .filter(|value: &String| !value.is_empty())
    };
    let base = variable(PICKER_BASE_VARIABLE)
        .as_deref()
        .and_then(launcher_base)
        .ok_or_else(|| failed(format!("{PICKER_BASE_VARIABLE} names no base key")))?;
    let target = variable(PICKER_TARGET_VARIABLE)
        .ok_or_else(|| failed(format!("{PICKER_TARGET_VARIABLE} names no pane")))?;
    let config = read_picker_config(&config_path()).map_err(failed)?;
    let offered = launcher_targets(&config.pickers, base);
    let layout = Layout::new(LauncherKeymap::LEGEND, &offered)
        .look(config.look)
        .theme(config.theme);
    let menu = Menu::new(&offered, &[], None, Fuzzy, ReadlineField::default());
    let mut screen = TerminalScreen::stdout(layout);
    let Some(chosen) =
        pick(menu, &LauncherKeymap, &mut TerminalInput, &mut screen).map_err(failed)?
    else {
        return Ok(());
    };
    if picker::is_bookmark(&chosen.label, &config.pickers) {
        eprintln!("{}", picker::bookmark_this_folder()?);
        return Ok(());
    }
    let binary = env::current_exe()
        .map_err(|failure| failed(format!("cannot find this binary: {failure}")))?;
    let popup = process::id().to_string();
    let arguments = ["deliver", &chosen.label, &target, AFTER_PID, &popup];
    spawn_detached(
        &binary,
        &arguments,
        Some(&spawned_jump_log_file(&state_dir())),
    )
    .map_err(|failure| failed(format!("cannot start {}: {failure}", chosen.label)))
}

#[cfg(test)]
mod tests;
