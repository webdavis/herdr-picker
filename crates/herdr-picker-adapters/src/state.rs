use std::path::{Path, PathBuf};

// The folder herdr gives the plugin, worked out the way herdr does when it is not named, so a
// shell key outside herdr reads what herdr's hooks wrote.
pub fn plugin_state_dir(
    state_dir: Option<&str>,
    xdg_state_home: Option<&str>,
    home: Option<&str>,
) -> PathBuf {
    let set = |value: Option<&str>| value.filter(|value| !value.is_empty()).map(PathBuf::from);
    set(state_dir).unwrap_or_else(|| {
        set(xdg_state_home)
            .unwrap_or_else(|| PathBuf::from(home.unwrap_or_default()).join(".local/state"))
            .join("herdr/plugins/herdr-picker")
    })
}
pub fn mru_file(state: &Path) -> PathBuf {
    state.join("mru")
}
pub fn spawned_jump_log_file(state: &Path) -> PathBuf {
    state.join("jump.log")
}
pub fn places_file(state: &Path) -> PathBuf {
    state.join("places")
}
pub fn snapshot_file(state: &Path) -> PathBuf {
    state.join("agents.json")
}
pub fn send_queue_file(state: &Path) -> PathBuf {
    state.join("send-queue.json")
}
pub fn bookmarks_file(state: &Path) -> PathBuf {
    state.join("bookmarks")
}
pub fn quickfix_file(state: &Path) -> PathBuf {
    state.join("quickfix.json")
}
pub fn captures_dir(state: &Path) -> PathBuf {
    state.join("captures")
}
pub fn picker_rows_file(state: &Path, run: u32) -> PathBuf {
    state.join(format!("picker-rows-{run}.jsonl"))
}
pub fn picker_result_file(state: &Path, run: u32) -> PathBuf {
    state.join(format!("picker-result-{run}"))
}

#[cfg(test)]
mod tests;
