use pwe_domain::test_engine::{TestEvent, TestStatus};

use crate::test_engine::{LibtestReader, read_libtest_listing};

const TWO_CRATES: &str = include_str!("../libtest/fixtures/cargo-test-two-crates.txt");

fn statuses(output: &str) -> Vec<(String, TestStatus)> {
    let mut reader = LibtestReader::new("cargo", "/work");
    let mut events: Vec<TestEvent> = output
        .lines()
        .flat_map(|line| reader.read_line(line))
        .collect();
    events.extend(reader.finish());
    events
        .into_iter()
        .filter_map(|event| match event {
            TestEvent::TestFinished(finished) => Some((finished.id.to_string(), finished.status)),
            _ => None,
        })
        .collect()
}

fn status_of(statuses: &[(String, TestStatus)], id: &str) -> Option<TestStatus> {
    statuses
        .iter()
        .find_map(|(found, status)| (found == id).then_some(*status))
}

#[test]
fn two_crates_with_a_same_named_test_get_two_different_ids() {
    let statuses = statuses(TWO_CRATES);

    assert_eq!(
        (
            status_of(&statuses, "cargo:crates/alpha/src/lib.rs::tests::same_name"),
            status_of(&statuses, "cargo:crates/beta/src/lib.rs::tests::same_name"),
        ),
        (Some(TestStatus::Passed), Some(TestStatus::Failed))
    );
}

#[test]
fn an_integration_test_file_is_named_from_the_repo_root() {
    let statuses = statuses(TWO_CRATES);

    assert_eq!(
        (
            status_of(&statuses, "cargo:crates/alpha/tests/it.rs::outside"),
            status_of(&statuses, "cargo:crates/beta/tests/it.rs::outside"),
        ),
        (Some(TestStatus::Passed), Some(TestStatus::Passed))
    );
}

#[test]
fn a_doc_test_in_a_workspace_is_named_from_the_repo_root() {
    assert_eq!(
        status_of(
            &statuses(TWO_CRATES),
            "cargo:crates/alpha/src/lib.rs::two[line 1]"
        ),
        Some(TestStatus::Passed)
    );
}

#[test]
fn an_artifact_line_a_test_prints_renames_no_test() {
    let spoofed = r#"{"reason":"compiler-artifact","target":{"src_path":"/work/elsewhere.rs"},"executable":"/work/target/probe-1"}"#;
    let output = [
        "     Running unittests src/lib.rs (/work/target/probe-1)",
        spoofed,
        "     Running unittests src/lib.rs (/work/target/probe-1)",
        "test tests::passes ... ok",
    ]
    .join("\n");

    assert_eq!(
        statuses(&output),
        vec![(
            "cargo:src/lib.rs::tests::passes".to_string(),
            TestStatus::Passed
        )]
    );
}

#[test]
fn the_listing_names_a_workspace_test_from_the_repo_root() {
    let artifacts = TWO_CRATES.lines().filter(|line| line.starts_with('{'));
    let listing = artifacts.chain([
        "     Running unittests src/lib.rs (target/debug/build/beta/3333333333333333/out/beta-3333333333333333)",
        "tests::same_name: test",
    ]);

    assert_eq!(
        read_libtest_listing("cargo", "/work", listing)
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        vec!["cargo:crates/beta/src/lib.rs::tests::same_name".to_string()]
    );
}
