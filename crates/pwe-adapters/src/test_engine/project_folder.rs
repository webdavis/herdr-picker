use std::path::{Path, PathBuf};

pub fn repo_root(folder: &Path) -> PathBuf {
    folder
        .ancestors()
        .find(|ancestor| ancestor.join(".git").exists())
        .unwrap_or(folder)
        .to_path_buf()
}

pub fn has_file(folder: &Path, name: &str) -> bool {
    folder.join(name).is_file()
}
