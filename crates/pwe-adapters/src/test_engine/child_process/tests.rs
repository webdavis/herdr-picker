use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use pwe_application::test_engine::{ToolProcess, ToolStartError};
use pwe_domain::test_engine::ToolEnd;

use super::*;
use crate::cli_command::spawning;

static NEXT_FOLDER: AtomicU64 = AtomicU64::new(0);

struct FakeBin(PathBuf);

impl FakeBin {
    fn with(name: &str, script: &str) -> Self {
        let number = NEXT_FOLDER.fetch_add(1, Ordering::Relaxed);
        let folder =
            std::env::temp_dir().join(format!("pwe-child-{}-{number}", std::process::id()));
        fs::create_dir_all(&folder).unwrap();
        let program = folder.join(name);
        let _spawning = spawning();
        fs::write(&program, format!("#!/bin/sh\n{script}")).unwrap();
        fs::set_permissions(&program, fs::Permissions::from_mode(0o700)).unwrap();
        FakeBin(folder)
    }

    fn path_variable(&self) -> Option<OsString> {
        Some(self.0.as_os_str().to_owned())
    }

    fn run(&self, name: &str) -> (Vec<String>, ToolEnd) {
        let mut tools = ChildProcess::new(self.path_variable());
        let mut run = {
            let _spawning = spawning();
            tools
                .start(&[name.to_string()], &self.0.to_string_lossy())
                .unwrap_or_else(|failure| panic!("{failure}"))
        };
        let mut lines = Vec::new();
        while let Some(line) = run.next_line() {
            lines.push(line);
        }
        (lines, run.wait())
    }
}

impl Drop for FakeBin {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn stdout_and_stderr_lines_keep_the_order_the_tool_wrote_them() {
    let bin = FakeBin::with("tool", "echo one\necho two >&2\necho three\nexit 101\n");

    assert_eq!(
        bin.run("tool"),
        (
            vec!["one".to_string(), "two".to_string(), "three".to_string()],
            ToolEnd::Exited { code: Some(101) }
        )
    );
}

#[test]
fn a_tool_killed_mid_run_ends_with_no_exit_code() {
    let bin = FakeBin::with("tool", "echo started\nkill -9 $$\n");

    assert_eq!(
        bin.run("tool"),
        (vec!["started".to_string()], ToolEnd::Exited { code: None })
    );
}

#[test]
fn the_tool_runs_in_its_own_process_group_and_its_folder() {
    let bin = FakeBin::with("tool", "ps -o pgid= -p $$ | tr -d ' '\necho $$\npwd -P\n");

    let (lines, _) = bin.run("tool");

    assert_eq!(lines[0], lines[1], "{lines:?}");
    assert_eq!(
        PathBuf::from(&lines[2]),
        bin.0.canonicalize().unwrap(),
        "{lines:?}"
    );
}

#[test]
fn a_program_missing_from_the_search_path_is_named() {
    let bin = FakeBin::with("tool", "");
    let mut tools = ChildProcess::new(bin.path_variable());

    let refused = tools.start(&["no-such-tool".to_string()], "/").err();

    assert_eq!(
        refused,
        Some(ToolStartError::Missing("no-such-tool".to_string()))
    );
}
