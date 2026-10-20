use std::path::PathBuf;

fn plugin_state_dir(state_dir: Option<&str>, home: Option<&str>) -> PathBuf {
    let directory = state_dir
        .filter(|directory| !directory.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            format!(
                "{}/.local/state/herdr/plugins/herdr-picker",
                home.unwrap_or_default()
            )
        });
    PathBuf::from(directory)
}
pub fn mru_file(state_dir: Option<&str>, home: Option<&str>) -> PathBuf {
    plugin_state_dir(state_dir, home).join("mru")
}
pub fn spawned_jump_log_file(state_dir: Option<&str>, home: Option<&str>) -> PathBuf {
    plugin_state_dir(state_dir, home).join("jump.log")
}
pub fn places_file(state_dir: Option<&str>, home: Option<&str>) -> PathBuf {
    plugin_state_dir(state_dir, home).join("places")
}
pub fn snapshot_file(state_dir: Option<&str>, home: Option<&str>) -> PathBuf {
    plugin_state_dir(state_dir, home).join("agents.json")
}
pub fn send_queue_file(state_dir: Option<&str>, home: Option<&str>) -> PathBuf {
    plugin_state_dir(state_dir, home).join("send-queue.json")
}
pub fn bookmarks_file(state_dir: Option<&str>, home: Option<&str>) -> PathBuf {
    plugin_state_dir(state_dir, home).join("bookmarks")
}
pub fn quickfix_file(state_dir: Option<&str>, home: Option<&str>) -> PathBuf {
    plugin_state_dir(state_dir, home).join("quickfix.json")
}
pub fn captures_dir(state_dir: Option<&str>, home: Option<&str>) -> PathBuf {
    plugin_state_dir(state_dir, home).join("captures")
}
pub fn picker_rows_file(state_dir: Option<&str>, home: Option<&str>, run: u32) -> PathBuf {
    plugin_state_dir(state_dir, home).join(format!("picker-rows-{run}.jsonl"))
}
pub fn picker_result_file(state_dir: Option<&str>, home: Option<&str>, run: u32) -> PathBuf {
    plugin_state_dir(state_dir, home).join(format!("picker-result-{run}"))
}

#[cfg(test)]
mod tests;
