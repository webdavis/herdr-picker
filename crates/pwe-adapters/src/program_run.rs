use std::ffi::OsStr;
use std::io::{self, Write};
use std::process::{Command, Output, Stdio};

use crate::program::on_path;

pub(crate) enum ProgramRunError {
    Missing,
    Unrunnable(io::Error),
}

pub(crate) fn run_on_path(
    name: &str,
    path_variable: Option<&OsStr>,
    arguments: &[&str],
    input: &str,
) -> Result<Output, ProgramRunError> {
    let program = on_path(name, path_variable).ok_or(ProgramRunError::Missing)?;
    let mut child = Command::new(program)
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(ProgramRunError::Unrunnable)?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input.as_bytes())
            .map_err(ProgramRunError::Unrunnable)?;
    }
    child
        .wait_with_output()
        .map_err(ProgramRunError::Unrunnable)
}
