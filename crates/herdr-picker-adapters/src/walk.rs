use std::ffi::OsStr;
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Entry {
    Files,
    Folders,
}

// Which entries under an absolute root to find; a depth of 1 is the root's own entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Walk {
    pub root: PathBuf,
    pub entry: Entry,
    pub hidden: bool,
    pub depth: Option<usize>,
}

// Stops at the first path the receiver refuses, so a list that closed stops the search.
pub fn walk(
    walk: &Walk,
    fd: Option<&Path>,
    mut found: impl FnMut(&Path) -> io::Result<()>,
) -> io::Result<()> {
    match fd {
        Some(fd) => with_fd(fd, walk, &mut found),
        None => read_folders(walk, &walk.root, 1, &mut found),
    }
}

fn with_fd(
    fd: &Path,
    walk: &Walk,
    found: &mut impl FnMut(&Path) -> io::Result<()>,
) -> io::Result<()> {
    let mut command = Command::new(fd);
    let kind = match walk.entry {
        Entry::Files => "f",
        Entry::Folders => "d",
    };
    // An absolute root makes fd print absolute paths, under the root as it was given.
    command.args(["--color=never", "--type", kind]);
    if walk.hidden {
        command.args(["--hidden", "--exclude", ".git"]);
    }
    if let Some(depth) = walk.depth {
        command.arg(format!("--max-depth={depth}"));
    }
    let mut child = command
        .arg(".")
        .arg(&walk.root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let paths = BufReader::new(child.stdout.take().expect("fd's stdout is piped"));
    let handed = paths.split(b'\n').try_for_each(|line| {
        let line = line?;
        // fd ends a folder with a slash.
        let path = line.strip_suffix(b"/").filter(|path| !path.is_empty());
        found(Path::new(OsStr::from_bytes(path.unwrap_or(&line))))
    });
    if handed.is_err() {
        let _ = child.kill();
    }
    let _ = child.wait();
    handed
}

fn read_folders(
    walk: &Walk,
    folder: &Path,
    level: usize,
    found: &mut impl FnMut(&Path) -> io::Result<()>,
) -> io::Result<()> {
    let Ok(entries) = fs::read_dir(folder) else {
        return Ok(());
    };
    let mut entries: Vec<_> = entries.filter_map(Result::ok).collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let name = entry.file_name();
        let dotted = name.as_bytes().starts_with(b".");
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if name == ".git" || (dotted && !walk.hidden) {
            continue;
        }
        let path = entry.path();
        let wanted = match walk.entry {
            Entry::Files => kind.is_file(),
            Entry::Folders => kind.is_dir(),
        };
        if wanted {
            found(&path)?;
        }
        if kind.is_dir() && walk.depth.is_none_or(|depth| level < depth) {
            read_folders(walk, &path, level + 1, found)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
