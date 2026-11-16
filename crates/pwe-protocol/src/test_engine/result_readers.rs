use pwe_domain::test_engine::{ResultReader, TestEvent, TestId};

use super::LibtestReader;
use super::cargo_build::BuildDiagnostics;

const LIBTEST: &str = "libtest";
pub const PARSERS: [&str; 1] = [LIBTEST];

struct CargoLibtestReader {
    build: BuildDiagnostics,
    tests: LibtestReader,
}

impl ResultReader for CargoLibtestReader {
    fn read_line(&mut self, line: &str) -> Vec<TestEvent> {
        match self.build.read(line) {
            Some(build_failed) => vec![build_failed],
            None => self.tests.read_line(line),
        }
    }

    fn finish(&mut self) -> Vec<TestEvent> {
        self.tests.finish()
    }
}

pub fn result_reader(parser: &str, runner: &str, repo_root: &str) -> Option<Box<dyn ResultReader>> {
    match parser {
        LIBTEST => Some(Box::new(CargoLibtestReader {
            build: BuildDiagnostics::new(repo_root),
            tests: LibtestReader::new(runner, repo_root),
        })),
        _ => None,
    }
}

pub fn tool_test_name(parser: &str, id: &TestId) -> Option<String> {
    match parser {
        LIBTEST if id.suite.is_empty() => Some(id.name.clone()),
        LIBTEST => Some(format!("{}::{}", id.suite, id.name)),
        _ => None,
    }
}
