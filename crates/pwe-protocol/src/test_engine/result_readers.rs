use pwe_domain::test_engine::{ResultReader, TestEvent, TestId};

use super::LibtestReader;

const LIBTEST: &str = "libtest";
pub const PARSERS: [&str; 1] = [LIBTEST];

impl ResultReader for LibtestReader {
    fn read_line(&mut self, line: &str) -> Vec<TestEvent> {
        LibtestReader::read_line(self, line)
    }

    fn finish(&mut self) -> Vec<TestEvent> {
        LibtestReader::finish(self)
    }
}

pub fn result_reader(parser: &str, runner: &str, repo_root: &str) -> Option<Box<dyn ResultReader>> {
    match parser {
        LIBTEST => Some(Box::new(LibtestReader::new(runner, repo_root))),
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
