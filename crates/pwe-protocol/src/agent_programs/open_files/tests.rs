use std::path::PathBuf;

use super::*;

#[test]
fn each_name_belongs_to_the_process_listed_before_it() {
    let printed = "p102\nfcwd\nn/work/project\nf12\nn/home/someone/.omp/run/session-owners/44444444.lock\np103\nf3\nn/tmp/other\n";

    assert_eq!(
        open_files_in_lsof_listing(printed),
        vec![
            (102, PathBuf::from("/work/project")),
            (
                102,
                PathBuf::from("/home/someone/.omp/run/session-owners/44444444.lock")
            ),
            (103, PathBuf::from("/tmp/other")),
        ]
    );
}

#[test]
fn names_before_any_process_are_dropped() {
    assert!(open_files_in_lsof_listing("n/tmp/x\npnot-a-number\nn/tmp/y\n").is_empty());
}
