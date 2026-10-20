use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use herdr_picker_protocol::{parse_header, parse_row};

use crate::builtin::Places;
use herdr_picker_domain::PickerRow;

// A git repository under the temp directory, removed when the test ends; git runs without the
// user's settings and with fixed dates, so its commits have the same hashes on every run.
pub(crate) struct GitRepo(pub(crate) PathBuf);

impl GitRepo {
    pub(crate) fn new(name: &str) -> Self {
        let temp = std::env::temp_dir()
            .canonicalize()
            .expect("the temp directory");
        let root = temp.join(format!("hwj-git-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("a repository folder");
        let repo = Self(root);
        repo.git(&["init", "-q", "-b", "main"]);
        repo
    }

    pub(crate) fn git(&self, arguments: &[&str]) -> String {
        git_in(&self.0, arguments)
    }

    pub(crate) fn write(&self, name: &str, text: &str) {
        let path = self.0.join(name);
        fs::create_dir_all(path.parent().expect("a parent")).expect("a folder");
        fs::write(path, text).expect("a file");
    }

    pub(crate) fn commit(&self, message: &str) {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", message]);
    }

    pub(crate) fn path(&self, name: &str) -> String {
        self.0.join(name).to_string_lossy().into_owned()
    }

    pub(crate) fn places(&self) -> Places {
        Places {
            cwd: self.0.clone(),
            home: None,
            fd: None,
            bat: None,
            bookmarks: self.0.join(".git/bookmarks"),
            quickfix: self.0.join(".git/quickfix.json"),
            captures: self.0.join(".git/captures"),
        }
    }
}

pub(crate) fn git_in(folder: &Path, arguments: &[&str]) -> String {
    let ran = Command::new("git")
        .arg("-C")
        .arg(folder)
        .args(arguments)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_AUTHOR_NAME", "Ada")
        .env("GIT_AUTHOR_EMAIL", "ada@example.com")
        .env("GIT_COMMITTER_NAME", "Ada")
        .env("GIT_COMMITTER_EMAIL", "ada@example.com")
        .env("GIT_AUTHOR_DATE", "2026-01-02T03:04:05Z")
        .env("GIT_COMMITTER_DATE", "2026-01-02T03:04:05Z")
        .output()
        .expect("git runs");
    assert!(
        ran.status.success(),
        "git {arguments:?}: {}",
        String::from_utf8_lossy(&ran.stderr)
    );
    String::from_utf8_lossy(&ran.stdout).trim_end().to_string()
}

impl Drop for GitRepo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

// What a picker printed: its header line, if any, and its rows.
pub(crate) fn read(printed: &str) -> (Option<String>, Vec<PickerRow>) {
    let mut header = None;
    let mut rows = Vec::new();
    for (index, line) in printed.lines().enumerate() {
        match parse_header(line) {
            Some(text) => header = Some(text),
            None => rows.push(parse_row(line, index + 1).expect("a row")),
        }
    }
    (header, rows)
}

pub(crate) fn labels(rows: &[PickerRow]) -> Vec<String> {
    rows.iter().map(|row| row.label.clone()).collect()
}
