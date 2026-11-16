use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use super::*;

struct Sandbox(PathBuf);

impl Sandbox {
    fn new(name: &str) -> Self {
        let directory =
            std::env::temp_dir().join(format!("hwj-rex-program-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("a sandbox directory");
        Self(directory)
    }

    fn program(&self, path: &str) -> PathBuf {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().expect("a parent")).expect("a folder");
        fs::write(&path, "#!/bin/sh\n").expect("a program");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("executable");
        path
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const IN_THE_BUNDLE: &str =
    "Applications/Rex Beta.app/Contents/Helpers/Rex Server.rex/Contents/MacOS/rex";

#[test]
fn rex_on_path_comes_first() {
    let sandbox = Sandbox::new("path");
    let on_path = sandbox.program("bin/rex");
    sandbox.program(IN_THE_BUNDLE);
    let path_variable = sandbox.0.join("bin").into_os_string();

    assert_eq!(
        rex_program(
            Some(&path_variable),
            &sandbox.0.join("Applications"),
            "Rex Beta"
        ),
        Ok(on_path)
    );
}

#[test]
fn without_rex_on_path_the_one_inside_the_named_app_is_used() {
    let sandbox = Sandbox::new("bundle");
    let bundled = sandbox.program(IN_THE_BUNDLE);

    assert_eq!(
        rex_program(None, &sandbox.0.join("Applications"), "Rex Beta"),
        Ok(bundled)
    );
}

#[test]
fn a_missing_rex_is_named_plainly() {
    let sandbox = Sandbox::new("missing");

    let missing = rex_program(None, &sandbox.0.join("Applications"), "Rex")
        .expect_err("no rex")
        .to_string();

    assert!(missing.contains("rex is not on PATH"), "{missing}");
    assert!(missing.contains("Rex.app"), "{missing}");
}
