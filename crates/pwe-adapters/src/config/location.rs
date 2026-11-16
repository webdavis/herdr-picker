use std::path::PathBuf;

pub fn config_file(
    plugin_config_dir: Option<&str>,
    xdg_config_home: Option<&str>,
    home: Option<&str>,
) -> PathBuf {
    let directory = match plugin_config_dir.filter(|directory| !directory.is_empty()) {
        Some(directory) => PathBuf::from(directory),
        None => base_config_dir(xdg_config_home, home).join("pwe"),
    };
    directory.join("config.toml")
}

fn base_config_dir(xdg_config_home: Option<&str>, home: Option<&str>) -> PathBuf {
    match xdg_config_home.filter(|directory| !directory.is_empty()) {
        Some(directory) => PathBuf::from(directory),
        None => PathBuf::from(home.unwrap_or_default()).join(".config"),
    }
}
