use std::fs;
use std::path::PathBuf;

use super::*;
use crate::program::on_path;

struct Tree(PathBuf);

impl Tree {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("hwj-walk-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        for file in [
            "a.txt",
            "src/main.rs",
            "src/deep/x/y.txt",
            ".hidden/secret",
            ".env",
            ".git/config",
        ] {
            let path = root.join(file);
            fs::create_dir_all(path.parent().expect("a parent")).expect("a folder");
            fs::write(&path, "x").expect("a file");
        }
        Self(root)
    }

    fn found(
        &self,
        entry: Entry,
        hidden: bool,
        depth: Option<usize>,
        fd: Option<&Path>,
    ) -> Vec<String> {
        let search = Walk {
            root: self.0.clone(),
            entry,
            hidden,
            depth,
        };
        let mut found = Vec::new();
        walk(&search, fd, |path| {
            let inside = path.strip_prefix(&self.0).expect("under the root");
            found.push(inside.to_string_lossy().into_owned());
            Ok(())
        })
        .expect("a walk");
        found.sort();
        found
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn installed_fd() -> Option<PathBuf> {
    let fd = on_path("fd", std::env::var_os("PATH").as_deref());
    if fd.is_none() {
        eprintln!("fd is not installed; only the walk without it is checked");
    }
    fd
}

#[test]
fn without_fd_files_are_found_by_reading_folders_skipping_dot_entries() {
    let tree = Tree::new("files");

    assert_eq!(
        tree.found(Entry::Files, false, None, None),
        ["a.txt", "src/deep/x/y.txt", "src/main.rs"]
    );
}

#[test]
fn without_fd_hidden_adds_dot_entries_but_never_the_git_folder() {
    let tree = Tree::new("hidden");

    assert_eq!(
        tree.found(Entry::Files, true, None, None),
        [
            ".env",
            ".hidden/secret",
            "a.txt",
            "src/deep/x/y.txt",
            "src/main.rs"
        ]
    );
}

#[test]
fn without_fd_a_depth_stops_that_many_levels_down_and_folders_can_be_listed() {
    let tree = Tree::new("depth");

    assert_eq!(
        tree.found(Entry::Files, false, Some(2), None),
        ["a.txt", "src/main.rs"]
    );
    assert_eq!(
        tree.found(Entry::Folders, false, None, None),
        ["src", "src/deep", "src/deep/x"]
    );
}

#[test]
fn fd_finds_the_same_entries_as_the_walk_without_it() {
    let tree = Tree::new("fd");
    let Some(fd) = installed_fd() else { return };

    for (entry, hidden, depth) in [
        (Entry::Files, false, None),
        (Entry::Files, true, None),
        (Entry::Files, false, Some(2)),
        (Entry::Folders, false, None),
    ] {
        assert_eq!(
            tree.found(entry, hidden, depth, Some(&fd)),
            tree.found(entry, hidden, depth, None),
            "{entry:?} hidden {hidden} depth {depth:?}"
        );
    }
}

#[test]
fn a_walk_stops_when_the_receiver_refuses_a_path() {
    let tree = Tree::new("stop");
    let search = Walk {
        root: tree.0.clone(),
        entry: Entry::Files,
        hidden: false,
        depth: None,
    };
    for fd in [None, installed_fd()] {
        let mut seen = 0;
        let stopped = walk(&search, fd.as_deref(), |_| {
            seen += 1;
            Err(io::Error::from(io::ErrorKind::BrokenPipe))
        });

        assert_eq!(
            stopped.map_err(|failure| failure.kind()),
            Err(io::ErrorKind::BrokenPipe)
        );
        assert_eq!(seen, 1);
    }
}
