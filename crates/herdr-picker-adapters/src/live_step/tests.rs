use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

use herdr_picker_protocol::PICKER_STATE_VARIABLE;

use super::*;
use crate::picker_files::{cached_rows, picker_error_file, read_step_header};

struct Sandbox(PathBuf);

impl Sandbox {
    fn new(name: &str) -> Self {
        let directory =
            std::env::temp_dir().join(format!("hwj-live-step-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("a sandbox directory");
        Self(directory)
    }

    fn command(&self, body: &str) -> Command {
        let path = self.0.join("picker.sh");
        fs::write(&path, format!("#!/bin/sh\n{body}\n")).expect("a picker script");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).expect("executable");
        let mut command = Command::new(path);
        command.arg("--from-test");
        command.env(PICKER_STATE_VARIABLE, "7");
        command
    }

    fn cache(&self) -> PathBuf {
        self.0.join("rows.jsonl")
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const ECHOES: &str = r#"printf '{"header":"%s %s"}\n{"label":"%s","enter":"none"}\n' "$1" "$HERDR_PICKER_STATE" "$HERDR_PICKER_QUERY""#;

#[test]
fn a_live_step_runs_its_command_again_with_the_query_and_its_own_args_and_state() {
    let sandbox = Sandbox::new("rerun");
    let cache = sandbox.cache();
    save_step_command(&cache, &sandbox.command(ECHOES)).expect("the command is saved");

    rerun_step(&cache, "fo o").expect("the step runs again");

    let labels: Vec<String> = cached_rows(&cache)
        .expect("rows")
        .into_iter()
        .map(|row| row.label)
        .collect();
    assert_eq!(labels, vec!["fo o"]);
    assert_eq!(read_step_header(&cache), Some("--from-test 7".to_string()));
}

#[test]
fn a_failing_rerun_leaves_its_error_for_the_footer() {
    let sandbox = Sandbox::new("fails");
    let cache = sandbox.cache();
    save_step_command(&cache, &sandbox.command("echo no repository >&2; exit 2"))
        .expect("the command is saved");

    assert!(rerun_step(&cache, "x").is_err());
    assert_eq!(
        fs::read_to_string(picker_error_file(&cache)).expect("an error"),
        "the picker command stopped with exit status: 2: no repository"
    );
}

#[test]
fn without_a_saved_command_there_is_nothing_to_run_again() {
    let sandbox = Sandbox::new("unsaved");

    assert!(rerun_step(&sandbox.cache(), "x").is_err());
}
