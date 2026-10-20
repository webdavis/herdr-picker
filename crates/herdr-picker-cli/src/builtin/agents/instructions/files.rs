use std::fs;
use std::path::{Path, PathBuf};

use herdr_picker_adapters::{Entry, Walk, walk};
use serde_json::Value;

use super::INSTRUCTION_NAMES;

// The files in a folder, by name; `skills` is every file, `entries` the named file in each subfolder.
pub(super) fn skills(folder: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = listed(folder)
        .into_iter()
        .filter(|path| path.is_file())
        .collect();
    files.sort();
    files
}

pub(super) fn entries(folder: &Path, name: &str) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = listed(folder)
        .into_iter()
        .map(|entry| entry.join(name))
        .filter(|path| path.is_file())
        .collect();
    files.sort();
    files
}

// The files in the folder whose text names the repository.
pub(super) fn mentioning(folder: &Path, repo: &str) -> Vec<PathBuf> {
    if repo.is_empty() {
        return Vec::new();
    }
    skills(folder)
        .into_iter()
        .filter(|path| fs::read_to_string(path).is_ok_and(|text| text.contains(repo)))
        .collect()
}

// Claude Code keeps a project's memory under its path with every other character a dash.
pub(in crate::builtin::agents) fn claude_project(home: &Path, root: &Path) -> PathBuf {
    let name: String = root
        .to_string_lossy()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .collect();
    home.join(".claude/projects").join(name)
}

// The installed Claude Code plugins that load here: every user plugin, and project ones for this repo.
pub(super) fn plugin_roots(home: &Path, root: &Path) -> Vec<PathBuf> {
    let installed = home.join(".claude/plugins/installed_plugins.json");
    let Some(listed) = fs::read(installed)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
    else {
        return Vec::new();
    };
    let here = root.to_string_lossy();
    let installs = listed["plugins"]
        .as_object()
        .into_iter()
        .flat_map(|plugins| plugins.values());
    installs
        .flat_map(|versions| versions.as_array().cloned().unwrap_or_default())
        .filter(|install| install["scope"] == "user" || install["projectPath"] == *here)
        .filter_map(|install| install["installPath"].as_str().map(PathBuf::from))
        .collect()
}

// Instruction files deeper in the repository, and its handoffs; fd skips what git ignores.
pub(super) fn repo_files(root: &Path, fd: Option<&Path>) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let (mut instructions, mut handoffs) = (Vec::new(), Vec::new());
    let every = Walk {
        root: root.to_path_buf(),
        entry: Entry::Files,
        hidden: true,
        depth: None,
    };
    let _ = walk(&every, fd, |path| {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if INSTRUCTION_NAMES.contains(&name.as_ref()) || name == "SKILL.md" {
            instructions.push(path.to_path_buf());
        } else if name.to_lowercase().contains("handoff") {
            handoffs.push(path.to_path_buf());
        }
        Ok(())
    });
    instructions.sort();
    handoffs.sort();
    (instructions, handoffs)
}

fn listed(folder: &Path) -> Vec<PathBuf> {
    fs::read_dir(folder)
        .into_iter()
        .flatten()
        .filter_map(|entry| Some(entry.ok()?.path()))
        .collect()
}
