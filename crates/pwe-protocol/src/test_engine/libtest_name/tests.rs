use pwe_domain::test_engine::{TestEvent, TestFinished, TestId};

use crate::test_engine::{LibtestReader, read_libtest_listing};

const LIB_TARGET: &str = "     Running unittests src/lib.rs (/work/target/debug/deps/probe-1)";

fn finished(lines: &[&str]) -> Vec<TestFinished> {
    let mut reader = LibtestReader::new("cargo", "/work");
    let mut events: Vec<TestEvent> = lines
        .iter()
        .flat_map(|line| reader.read_line(line))
        .collect();
    events.extend(reader.finish());
    events
        .into_iter()
        .filter_map(|event| match event {
            TestEvent::TestFinished(finished) => Some(finished),
            _ => None,
        })
        .collect()
}

fn id(text: &str) -> TestId {
    text.parse().unwrap()
}

#[test]
fn a_test_name_with_an_escape_sequence_is_read_without_it() {
    let tests = finished(&[LIB_TARGET, "test tests::\u{1b}[31mred ... ok"]);

    assert_eq!(tests[0].id, id("cargo:src/lib.rs::tests::[31mred"));
}

#[test]
fn a_listed_test_name_with_an_escape_sequence_is_read_without_it() {
    let listed = read_libtest_listing(
        "cargo",
        "/work",
        [LIB_TARGET, "tests::\u{1b}[31mred\u{7}: test"].into_iter(),
    );

    assert_eq!(listed, [id("cargo:src/lib.rs::tests::[31mred")]);
}

#[test]
fn a_target_file_with_a_control_character_is_read_without_it() {
    let tests = finished(&[
        "     Running unittests src/l\u{7}ib.rs (/work/target/debug/deps/probe-1)",
        "test tests::passes ... ok",
    ]);

    assert_eq!(tests[0].id.file, "src/lib.rs");
}

#[test]
fn a_panic_place_with_an_escape_sequence_is_read_without_it() {
    let tests = finished(&[
        LIB_TARGET,
        "test tests::fails ... FAILED",
        "failures:",
        "---- tests::fails stdout ----",
        "thread 'tests::fails' panicked at src/\u{1b}[2Klib.rs:5:9:",
        "boom",
        "failures:",
    ]);

    let failure = tests[0]
        .failure
        .clone()
        .unwrap_or_else(|| panic!("{tests:?}"));
    assert_eq!(failure.file.as_deref(), Some("src/[2Klib.rs"));
}
