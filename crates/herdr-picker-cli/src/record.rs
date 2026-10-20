use std::env;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use herdr_picker_adapters::{
    FileWorkspaceHistory, SocketPanes, mru_file, places_file, record_place,
};
use herdr_picker_application::record;
use herdr_picker_domain::{Place, RECENT_PLACES};
use herdr_picker_protocol::{SOCKET_PATH_VARIABLE, parse_focused_id, parse_focused_pane};

use crate::command::state_dir;
use crate::picker_popup::popup_socket;

const PLACE_LOOKUP_CAP: Duration = Duration::from_millis(500);

fn state_file(file: fn(&Path) -> PathBuf) -> PathBuf {
    file(&state_dir())
}

pub(crate) fn mru_path() -> PathBuf {
    state_file(mru_file)
}

// The focus hooks: a workspace focus moves the last-workspace toggle, a pane focus is logged
// as a recent place.
pub(crate) fn run(event: &str) {
    record_at(&mru_path(), event);
    let socket = popup_socket(env::var(SOCKET_PATH_VARIABLE).ok());
    if let (Some(pane_id), Some(socket)) = (parse_focused_pane(event), socket) {
        let at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |since| since.as_secs());
        remember_pane(&socket, &pane_id, &state_file(places_file), at);
    }
}

pub(crate) fn record_at(state: &Path, event: &str) {
    if let Some(new_id) = parse_focused_id(event) {
        record(&mut FileWorkspaceHistory::at(state), &new_id);
    }
}

pub(crate) fn remember_pane(socket: &Path, pane_id: &str, places: &Path, at: u64) {
    let Ok(pane) = SocketPanes::at(socket, PLACE_LOOKUP_CAP).pane(pane_id) else {
        return;
    };
    let place = Place {
        at,
        workspace_id: pane.workspace_id,
        tab_id: pane.tab_id,
        pane_id: pane.pane_id,
        cwd: pane.cwd,
    };
    record_place(places, place, RECENT_PLACES);
}

#[cfg(test)]
mod tests;
