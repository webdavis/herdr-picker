use std::fs;
use std::path::{Path, PathBuf};

// The branch checked out where an agent works, and whether that folder is a linked worktree,
// read from git's own files so a long list runs no git.
pub(super) fn branch(cwd: &Path) -> Option<(String, bool)> {
    let dot_git = cwd
        .ancestors()
        .map(|folder| folder.join(".git"))
        .find(|path| path.exists())?;
    let (git_dir, worktree) = match dot_git.is_file() {
        true => (linked(&dot_git)?, true),
        false => (dot_git, false),
    };
    let head = fs::read_to_string(git_dir.join("HEAD")).ok()?;
    let name = match head.trim().strip_prefix("ref: refs/heads/") {
        Some(branch) => branch.to_string(),
        None => head.trim().chars().take(7).collect(),
    };
    Some((name, worktree))
}

fn linked(dot_git: &Path) -> Option<PathBuf> {
    let text = fs::read_to_string(dot_git).ok()?;
    let target = Path::new(text.trim().strip_prefix("gitdir:")?.trim());
    Some(dot_git.parent()?.join(target))
}
