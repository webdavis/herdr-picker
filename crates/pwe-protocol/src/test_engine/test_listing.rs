use pwe_domain::test_engine::{Diagnostic, TestEvent, TestId};
use serde_json::{Value, json};

use super::cargo_build::BuildDiagnostics;
use super::event_fields::diagnostic;
use super::{event_text, read_libtest_listing};

const LIBTEST: &str = "libtest";
pub const LIST_PARSERS: [&str; 1] = [LIBTEST];
const NOTHING_LISTED: &str = "No tests were listed.\n";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listing {
    pub tests: Vec<TestId>,
    pub build_failed: Option<Vec<Diagnostic>>,
}

pub fn read_listing(
    parser: &str,
    runner: &str,
    repo_root: &str,
    lines: &[String],
) -> Option<Listing> {
    if parser != LIBTEST {
        return None;
    }
    let mut build = BuildDiagnostics::new(repo_root);
    let build_failed = lines.iter().find_map(|line| match build.read(line) {
        Some(TestEvent::BuildFailed { diagnostics }) => Some(diagnostics),
        _ => None,
    });
    Some(Listing {
        tests: read_libtest_listing(runner, repo_root, lines.iter().map(String::as_str)),
        build_failed,
    })
}

pub fn test_json(id: &TestId) -> Value {
    json!({
        "id": id.to_string(),
        "runner": id.runner,
        "file": id.file,
        "suite": id.suite,
        "name": id.name,
        "case": id.case,
        "line": Value::Null,
    })
}

pub fn listing_json(listing: &Listing) -> Value {
    let mut document = json!({ "tests": listing.tests.iter().map(test_json).collect::<Vec<_>>() });
    if let Some(diagnostics) = &listing.build_failed {
        document["diagnostics"] = diagnostics
            .iter()
            .map(|found| Value::from(diagnostic(found)))
            .collect();
    }
    document
}

pub fn listing_text(listing: &Listing) -> String {
    if let Some(diagnostics) = &listing.build_failed {
        return event_text(&TestEvent::BuildFailed {
            diagnostics: diagnostics.clone(),
        });
    }
    if listing.tests.is_empty() {
        return NOTHING_LISTED.to_string();
    }
    listing.tests.iter().map(|id| format!("{id}\n")).collect()
}
