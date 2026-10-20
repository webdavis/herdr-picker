use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{env, fs, process};

use herdr_picker_adapters::{
    DEADLINE, Placement, SocketPopupHost, await_pane_result, picker_result_file, write_pane_state,
};
use herdr_picker_application::{JumpError, PopupHost};
use herdr_picker_domain::preview_popup_size;
use herdr_picker_protocol::{PICKER_RESULT_VARIABLE, PLUGIN_ID, PaneState, PopupPane};

use crate::command::{CommandError, failed};
use crate::picker;

const START_WITHIN: Duration = Duration::from_secs(5);

pub(crate) fn popup_socket(socket_path: Option<String>) -> Option<PathBuf> {
    socket_path
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
}

// herdr refuses when this plugin is not linked or another popup is open: run in place.
pub(crate) fn pick(
    socket: &Path,
    name: &str,
    config: &Path,
) -> Result<Option<String>, CommandError> {
    in_popup(socket, name, config, |_| {
        picker::pick(name, config, Placement::Inline)
    })
}

pub(crate) fn in_popup(
    socket: &Path,
    name: &str,
    config: &Path,
    refused: impl FnOnce(JumpError) -> Result<Option<String>, CommandError>,
) -> Result<Option<String>, CommandError> {
    let result = picker_result_file(
        env::var("HERDR_PLUGIN_STATE_DIR").ok().as_deref(),
        env::var("HOME").ok().as_deref(),
        process::id(),
    );
    if let Some(directory) = result.parent() {
        fs::create_dir_all(directory).map_err(failed)?;
    }
    let _ = fs::remove_file(&result);
    let cwd = env::current_dir().ok();
    let pane = PopupPane::configured_picker(
        name,
        &config.to_string_lossy(),
        &result.to_string_lossy(),
        cwd.as_deref().map(Path::to_string_lossy).as_deref(),
    );
    if let Err(refusal) = open(socket, pane) {
        return refused(refusal);
    }
    let answer = await_pane_result(&result, START_WITHIN);
    let _ = fs::remove_file(&result);
    answer.map_err(failed)
}

fn open(socket: &Path, pane: PopupPane) -> Result<(), JumpError> {
    let mut host = SocketPopupHost::at(socket, DEADLINE, PLUGIN_ID, pane);
    let window = host.window_size()?;
    host.open(preview_popup_size(window))
}

pub(crate) fn answer_in_popup(
    result: Option<PathBuf>,
    name: &str,
    config: &Path,
) -> Result<(), CommandError> {
    let result = &result.ok_or_else(|| {
        failed(format!(
            "{PICKER_RESULT_VARIABLE} is unset, so herdr did not run this as the plugin's picker popup"
        ))
    })?;
    write_pane_state(result, &PaneState::Running { pid: process::id() }).map_err(failed)?;
    let answer = match picker::pick(name, config, Placement::Popup) {
        Ok(instruction) => PaneState::Finished { instruction },
        Err(CommandError::Failed(reason)) => PaneState::Failed { reason },
        Err(CommandError::Usage) => PaneState::Failed {
            reason: crate::command::USAGE.to_string(),
        },
    };
    write_pane_state(result, &answer).map_err(failed)
}

#[cfg(test)]
mod tests;
