use pwe_domain::test_engine::{MatchedRunner, ToolEnd};

use super::{ToolProcess, ToolStartError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListTestsError {
    NoListCommand,
    Start(ToolStartError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolOutput {
    pub lines: Vec<String>,
    pub end: ToolEnd,
}

pub fn list_tests(
    runner: &MatchedRunner,
    tools: &mut dyn ToolProcess,
    cwd: &str,
) -> Result<ToolOutput, ListTestsError> {
    let list = runner
        .spec
        .list
        .as_ref()
        .ok_or(ListTestsError::NoListCommand)?;
    let argv: Vec<String> = runner
        .command_prefix
        .iter()
        .chain(&list.argv)
        .cloned()
        .collect();
    let mut run = tools.start(&argv, cwd).map_err(ListTestsError::Start)?;
    let mut lines = Vec::new();
    while let Some(line) = run.next_line() {
        lines.push(line);
    }
    Ok(ToolOutput {
        lines,
        end: run.wait(),
    })
}
