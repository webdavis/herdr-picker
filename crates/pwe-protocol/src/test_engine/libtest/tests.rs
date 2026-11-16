use pwe_domain::test_engine::{
    Counts, Failure, Stream, TestEvent, TestFinished, TestId, TestStatus, ToolEnd, UntrustedText,
    verdict,
};

use super::*;
use crate::test_engine::read_libtest_listing;

const WORKSPACE: &str = include_str!("fixtures/cargo-test-workspace.txt");
const DOC_TESTS: &str = include_str!("fixtures/cargo-test-doc.txt");
const LISTING: &str = include_str!("fixtures/cargo-test-list.txt");

fn read_all(output: &str) -> Vec<TestEvent> {
    let mut reader = LibtestReader::new("cargo");
    let mut events: Vec<TestEvent> = output
        .lines()
        .flat_map(|line| reader.read_line(line))
        .collect();
    events.extend(reader.finish());
    events
}

fn id(text: &str) -> TestId {
    text.parse().unwrap()
}

fn finished_of(events: &[TestEvent], test: &str) -> TestFinished {
    events
        .iter()
        .find_map(|event| match event {
            TestEvent::TestFinished(finished) if finished.id == id(test) => Some(finished.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no test-finished for {test}"))
}

fn untrusted(text: &str) -> Option<UntrustedText> {
    Some(UntrustedText::new(text))
}

fn in_the_lib_target(reader: &mut LibtestReader) {
    reader.read_line("     Running unittests src/lib.rs (/work/target/debug/deps/probe-1)");
}

#[test]
fn a_failed_assert_eq_reports_its_file_line_and_left_and_right() {
    let fails = finished_of(&read_all(WORKSPACE), "cargo:src/lib.rs::tests::fails");

    assert_eq!(
        (fails.status, fails.failure),
        (
            TestStatus::Failed,
            Some(Failure {
                message: UntrustedText::new("assertion `left == right` failed: two is two"),
                file: Some("src/lib.rs".to_string()),
                line: Some(5),
                expected: untrusted("3"),
                actual: untrusted("2"),
            })
        )
    );
}

#[test]
fn a_passing_test_finishes_the_moment_its_line_arrives() {
    let mut reader = LibtestReader::new("cargo");
    in_the_lib_target(&mut reader);

    let events = reader.read_line("test tests::passes ... ok");

    assert_eq!(
        events,
        vec![TestEvent::TestFinished(TestFinished {
            id: id("cargo:src/lib.rs::tests::passes"),
            status: TestStatus::Passed,
            duration_ms: None,
            file: None,
            line: None,
            attempt: 1,
            failure: None,
        })]
    );
}

#[test]
fn a_failed_test_waits_for_its_failure_block() {
    let mut reader = LibtestReader::new("cargo");
    in_the_lib_target(&mut reader);

    assert!(reader.read_line("test tests::fails ... FAILED").is_empty());
}

#[test]
fn an_ignored_test_is_skipped() {
    let skipped = finished_of(&read_all(WORKSPACE), "cargo:src/lib.rs::tests::skipped");

    assert_eq!(
        (skipped.status, skipped.failure),
        (TestStatus::Skipped, None)
    );
}

#[test]
fn a_panic_that_printed_output_reports_the_output_then_the_failure() {
    let events = read_all(WORKSPACE);
    let prints = id("cargo:src/lib.rs::tests::prints");
    let output_at = events.iter().position(|event| {
        *event
            == TestEvent::Output {
                id: Some(prints.clone()),
                stream: Stream::Stdout,
                text: UntrustedText::new("hello\n"),
            }
    });
    let finished_at = events.iter().position(
        |event| matches!(event, TestEvent::TestFinished(finished) if finished.id == prints),
    );
    let failure = finished_of(&events, "cargo:src/lib.rs::tests::prints").failure;

    assert!(output_at.is_some() && output_at < finished_at);
    assert_eq!(
        failure.map(|failure| (failure.message.text, failure.line, failure.expected)),
        Some(("boom".to_string(), Some(7), None))
    );
}

#[test]
fn each_test_binary_names_its_own_target_file() {
    let outside = finished_of(&read_all(WORKSPACE), "cargo:tests/it.rs::outside");

    assert_eq!(outside.status, TestStatus::Passed);
}

#[test]
fn the_whole_run_counts_every_test_cargo_ran() {
    let events = read_all(WORKSPACE);

    assert_eq!(
        verdict(&events, ToolEnd::Exited { code: Some(101) }).counts,
        Counts {
            passed: 2,
            failed: 2,
            skipped: 1,
            errored: 0,
        }
    );
}

#[test]
fn a_failed_doc_test_points_at_its_line_in_the_source_file() {
    let events = read_all(DOC_TESTS);
    let failed = finished_of(&events, "cargo:src/lib.rs::two[line 5]");

    assert_eq!(
        failed.failure.map(|failure| (
            failure.file,
            failure.line,
            failure.actual,
            failure.expected
        )),
        Some((
            Some("src/lib.rs".to_string()),
            Some(5),
            untrusted("2"),
            untrusted("3")
        ))
    );
    assert_eq!(
        finished_of(&events, "cargo:src/lib.rs::two[line 1]").status,
        TestStatus::Passed
    );
}

#[test]
fn control_characters_in_a_failure_are_stripped_and_counted() {
    let mut reader = LibtestReader::new("cargo");
    in_the_lib_target(&mut reader);
    let lines = [
        "test tests::loud ... FAILED",
        "",
        "failures:",
        "",
        "---- tests::loud stdout ----",
        "",
        "thread 'tests::loud' (1) panicked at src/lib.rs:9:5:",
        "red \u{1b}[31malert\u{7}",
        "",
        "failures:",
        "    tests::loud",
    ];

    let events: Vec<TestEvent> = lines
        .iter()
        .flat_map(|line| reader.read_line(line))
        .collect();
    let message = finished_of(&events, "cargo:src/lib.rs::tests::loud")
        .failure
        .map(|failure| failure.message);

    assert_eq!(
        message,
        Some(UntrustedText {
            text: "red [31malert".to_string(),
            stripped: 2,
        })
    );
}

#[test]
fn a_failed_test_whose_block_never_came_finishes_failed_at_the_end() {
    let mut reader = LibtestReader::new("cargo");
    in_the_lib_target(&mut reader);
    reader.read_line("test tests::fails ... FAILED");

    let failed = finished_of(&reader.finish(), "cargo:src/lib.rs::tests::fails");

    assert_eq!((failed.status, failed.failure), (TestStatus::Failed, None));
}

#[test]
fn the_listing_names_every_test_of_every_target() {
    assert_eq!(
        read_libtest_listing("cargo", LISTING.lines()),
        vec![
            id("cargo:src/lib.rs::tests::fails"),
            id("cargo:src/lib.rs::tests::passes"),
            id("cargo:src/lib.rs::tests::prints"),
            id("cargo:src/lib.rs::tests::skipped"),
            id("cargo:tests/it.rs::outside"),
        ]
    );
}

#[test]
fn a_should_panic_test_keeps_its_plain_name() {
    let mut reader = LibtestReader::new("cargo");
    in_the_lib_target(&mut reader);

    let events = reader.read_line("test tests::refuses - should panic ... ok");

    assert_eq!(
        finished_of(&events, "cargo:src/lib.rs::tests::refuses").status,
        TestStatus::Passed
    );
}
