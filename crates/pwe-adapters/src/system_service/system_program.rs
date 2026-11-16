use std::io;
use std::process::Command;

use pwe_application::SystemServiceError;

pub(crate) fn run_system_program(
    program: &str,
    arguments: &[&str],
) -> Result<(), SystemServiceError> {
    let output = Command::new(program)
        .args(arguments)
        .output()
        .map_err(|error| match error.kind() {
            io::ErrorKind::NotFound => SystemServiceError::ProgramMissing(program.to_string()),
            _ => SystemServiceError::ProgramRefused {
                program: program.to_string(),
                detail: error.to_string(),
            },
        })?;
    if output.status.success() {
        return Ok(());
    }
    Err(SystemServiceError::ProgramRefused {
        program: program.to_string(),
        detail: format!(
            "{} ({})",
            String::from_utf8_lossy(&output.stderr).trim(),
            output.status
        ),
    })
}

pub(crate) fn unwritable(error: io::Error) -> SystemServiceError {
    SystemServiceError::JobUnwritable(error.to_string())
}
