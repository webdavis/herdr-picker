use std::env;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use herdr_picker_adapters::{
    AgentSnapshot, SocketPanes, drain_queue, read_snapshot, send_queue_file, snapshot_file,
    write_snapshot,
};
use herdr_picker_application::JumpError;
use herdr_picker_domain::carry_since;
use herdr_picker_protocol::{SOCKET_PATH_VARIABLE, parse_status_change};

use crate::command::state_dir;
use crate::picker_popup::popup_socket;

const HOOK_CAP: Duration = Duration::from_secs(2);

fn state_file(file: fn(&Path) -> PathBuf) -> PathBuf {
    file(&state_dir())
}

pub(crate) fn snapshot_path() -> PathBuf {
    state_file(snapshot_file)
}

pub(crate) fn queue_path() -> PathBuf {
    state_file(send_queue_file)
}

pub(crate) fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs())
}

// The `pane.agent_status_changed` hook: save what herdr runs now, then hand an agent that just
// finished its turn the prompt waiting for it.
pub(crate) fn run(event: &str) {
    let (Some((pane_id, status)), Some(socket)) = (
        parse_status_change(event),
        popup_socket(env::var(SOCKET_PATH_VARIABLE).ok()),
    ) else {
        return;
    };
    let mut panes = SocketPanes::at(&socket, HOOK_CAP);
    let _ = refresh(&mut panes, &snapshot_path(), &pane_id, now());
    let _ = drain_queue(&mut panes, &queue_path(), &pane_id, status);
}

// herdr's agents now, each keeping the clock the saved list gave it unless `changed` is the one
// whose status just changed.
pub(crate) fn refresh(
    panes: &mut SocketPanes,
    saved: &Path,
    changed: &str,
    now: u64,
) -> Result<AgentSnapshot, JumpError> {
    let before = read_snapshot(saved, Duration::MAX, SystemTime::now()).unwrap_or_default();
    let mut snapshot = panes.snapshot()?;
    snapshot.agents = carry_since(snapshot.agents, &before.agents, changed, now);
    let _ = write_snapshot(saved, &snapshot);
    Ok(snapshot)
}

#[cfg(test)]
mod tests;
