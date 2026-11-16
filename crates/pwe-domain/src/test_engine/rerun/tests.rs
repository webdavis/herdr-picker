use super::*;
use crate::test_engine::{TestFinished, TestStatus};

fn finished(name: &str, status: TestStatus, attempt: u32) -> TestEvent {
    TestEvent::TestFinished(TestFinished {
        id: id(name),
        status,
        duration_ms: None,
        file: None,
        line: None,
        attempt,
        failure: None,
    })
}

fn id(name: &str) -> TestId {
    format!("cargo:src/lib.rs::tests::{name}").parse().unwrap()
}

#[test]
fn the_tests_that_failed_or_errored_are_rerun_once_each_in_run_order() {
    let events = [
        finished("errors", TestStatus::Errored, 1),
        finished("passes", TestStatus::Passed, 1),
        finished("fails", TestStatus::Failed, 1),
        finished("skipped", TestStatus::Skipped, 1),
        finished("fails", TestStatus::Failed, 2),
    ];

    assert_eq!(failed_ids(&events), [id("errors"), id("fails")]);
}

#[test]
fn a_green_run_leaves_nothing_to_rerun() {
    let events = [finished("passes", TestStatus::Passed, 1)];

    assert!(failed_ids(&events).is_empty());
}
