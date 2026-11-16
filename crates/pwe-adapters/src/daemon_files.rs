use std::path::PathBuf;

const SYSTEM_JOB_NAME: &str = "pwe-daemon";

fn set(value: Option<&str>) -> Option<PathBuf> {
    value.filter(|value| !value.is_empty()).map(PathBuf::from)
}

pub fn pwe_state_folder(state_home: Option<&str>, home: Option<&str>) -> PathBuf {
    set(state_home)
        .unwrap_or_else(|| PathBuf::from(home.unwrap_or_default()).join(".local/state"))
        .join("pwe")
}

pub fn daemon_socket_file(
    runtime_dir: Option<&str>,
    state_home: Option<&str>,
    home: Option<&str>,
) -> PathBuf {
    set(runtime_dir)
        .map(|runtime| runtime.join("pwe"))
        .unwrap_or_else(|| pwe_state_folder(state_home, home))
        .join("daemon.sock")
}

pub fn daemon_log_file(state_home: Option<&str>, home: Option<&str>) -> PathBuf {
    pwe_state_folder(state_home, home).join("daemon.log")
}

pub fn launch_agent_file(home: Option<&str>) -> PathBuf {
    PathBuf::from(home.unwrap_or_default())
        .join("Library/LaunchAgents")
        .join(format!("{SYSTEM_JOB_NAME}.plist"))
}

pub fn systemd_unit_file(config_home: Option<&str>, home: Option<&str>) -> PathBuf {
    set(config_home)
        .unwrap_or_else(|| PathBuf::from(home.unwrap_or_default()).join(".config"))
        .join("systemd/user")
        .join(format!("{SYSTEM_JOB_NAME}.service"))
}

#[cfg(test)]
mod tests;
