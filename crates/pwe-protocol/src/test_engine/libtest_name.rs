use pwe_domain::test_engine::TestId;

use super::libtest_target::{TestTarget, TestTargets};

const SHOULD_PANIC: &str = " - should panic";

pub(super) fn test_id(runner: &str, target: &TestTarget, name: &str) -> Option<TestId> {
    match target {
        TestTarget::Unknown => None,
        TestTarget::File(file) => {
            let path = name.strip_suffix(SHOULD_PANIC).unwrap_or(name);
            Some(id_in(runner, file, path, None))
        }
        TestTarget::DocTests => {
            let (file, path, line) = doc_test(name)?;
            Some(id_in(runner, file, path, Some(format!("line {line}"))))
        }
    }
}

pub(super) fn doc_test_place(target: &TestTarget, name: &str) -> Option<(String, u32)> {
    match target {
        TestTarget::DocTests => doc_test(name).map(|(file, _, line)| (file.to_string(), line)),
        _ => None,
    }
}

fn doc_test(name: &str) -> Option<(&str, &str, u32)> {
    let (file, rest) = name.split_once(" - ")?;
    let (path, line) = rest.strip_suffix(')')?.rsplit_once(" (line ")?;
    Some((file, path, line.parse().ok()?))
}

fn id_in(runner: &str, file: &str, path: &str, case: Option<String>) -> TestId {
    let (suite, name) = path.rsplit_once("::").unwrap_or(("", path));
    TestId {
        runner: runner.to_string(),
        file: file.to_string(),
        suite: suite.to_string(),
        name: name.to_string(),
        case,
    }
}

pub fn read_libtest_listing<'a>(
    runner: &str,
    repo_root: &str,
    lines: impl Iterator<Item = &'a str>,
) -> Vec<TestId> {
    let mut targets = TestTargets::new(repo_root);
    let mut target = TestTarget::Unknown;
    let mut ids = Vec::new();
    for line in lines {
        if let Some(next) = targets.read(line) {
            target = next;
        } else if let Some(name) = line.strip_suffix(": test") {
            ids.extend(test_id(runner, &target, name));
        }
    }
    ids
}
