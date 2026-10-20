use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use super::*;

struct Sandbox(PathBuf);

impl Sandbox {
    fn new(name: &str) -> Self {
        let directory =
            std::env::temp_dir().join(format!("hwj-program-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("a sandbox directory");
        Self(directory)
    }

    fn program(&self, folder: &str, name: &str) -> PathBuf {
        let path = self.0.join(folder).join(name);
        fs::create_dir_all(path.parent().expect("a parent")).expect("a folder");
        fs::write(&path, "#!/bin/sh\n").expect("a program");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("executable");
        path
    }

    fn path_variable(&self, folders: &[&str]) -> std::ffi::OsString {
        std::env::join_paths(folders.iter().map(|folder| self.0.join(folder))).expect("a PATH")
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn on_path_finds_the_first_executable_of_that_name() {
    let sandbox = Sandbox::new("on-path");
    fs::create_dir_all(sandbox.0.join("empty")).expect("a folder");
    let first = sandbox.program("bin", "fd");
    sandbox.program("later", "fd");

    let path = sandbox.path_variable(&["empty", "bin", "later"]);

    assert_eq!(on_path("fd", Some(&path)), Some(first));
    assert_eq!(on_path("bat", Some(&path)), None);
    assert_eq!(on_path("fd", None), None);
}

#[test]
fn an_editor_named_by_full_path_reads_as_its_bare_name_when_path_finds_the_same_program() {
    let sandbox = Sandbox::new("bare");
    let nvim = sandbox.program("bin", "nvim");
    sandbox.program("other", "nvim");
    let editor = format!("{} -u 'my init.lua'", nvim.display());

    assert_eq!(
        bare_program(&editor, Some(&sandbox.path_variable(&["bin"]))),
        "nvim -u 'my init.lua'"
    );
    assert_eq!(
        bare_program(&editor, Some(&sandbox.path_variable(&["other", "bin"]))),
        editor
    );
    assert_eq!(
        bare_program("nvim", Some(&sandbox.path_variable(&["bin"]))),
        "nvim"
    );
}
