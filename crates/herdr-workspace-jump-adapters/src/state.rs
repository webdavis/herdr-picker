use std::path::PathBuf;

fn plugin_state_dir(state_dir: Option<&str>, home: Option<&str>) -> PathBuf {
    let directory = state_dir
        .filter(|directory| !directory.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            format!(
                "{}/.local/state/herdr/plugins/herdr-workspace-jump",
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

#[cfg(test)]
mod tests;
