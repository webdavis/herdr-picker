use std::path::PathBuf;

use pwe_application::{OpenFile, OpenFiles};

use crate::cli_command::{RecordedCli, spawning};

use super::*;

#[test]
fn the_files_a_command_holds_open_are_read_from_lsof() {
    let _guard = spawning();
    let lsof = RecordedCli::printing(
        "lsof",
        "p102\nf12\nn/home/someone/.omp/run/session-owners/one.lock\n",
    );
    let path = std::env::join_paths([lsof.folder()]).expect("a PATH");

    let files = LsofOpenFiles::on(Some(&path)).opened_by("omp");

    assert_eq!(
        files,
        vec![OpenFile {
            pid: 102,
            path: PathBuf::from("/home/someone/.omp/run/session-owners/one.lock"),
        }]
    );
    assert_eq!(lsof.calls(), vec![vec!["-w", "-F", "pn", "-c", "omp"]]);
}

#[test]
fn without_lsof_no_file_is_open() {
    assert!(LsofOpenFiles::on(None).opened_by("omp").is_empty());
}
