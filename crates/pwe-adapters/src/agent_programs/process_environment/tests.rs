use std::fs;

use crate::cli_command::{RecordedCli, spawning};

use super::*;

#[test]
fn a_linux_environ_file_is_read_without_ps() {
    let _guard = spawning();
    let ps = RecordedCli::printing("ps", "  PID CMD\n7 x CLAUDE_CONFIG_DIR=/from/ps\n");
    let proc_folder = ps.folder().join("proc");
    fs::create_dir_all(proc_folder.join("7")).expect("a process folder");
    fs::write(
        proc_folder.join("7/environ"),
        b"CLAUDE_CONFIG_DIR=/from/proc\0LANG=C\0",
    )
    .expect("an environ file");
    let path = std::env::join_paths([ps.folder()]).expect("a PATH");

    let environment = SystemProcessEnvironment::on(Some(&path), &proc_folder);

    assert_eq!(
        environment.variable(7, "CLAUDE_CONFIG_DIR").as_deref(),
        Some("/from/proc")
    );
}

#[test]
fn without_ps_or_proc_the_variable_is_absent() {
    let nowhere = std::env::temp_dir().join("pwe-no-proc-folder");
    let environment = SystemProcessEnvironment::on(None, &nowhere);

    assert_eq!(environment.variable(7, "CLAUDE_CONFIG_DIR"), None);
}
