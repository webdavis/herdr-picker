use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use pwe_application::JumpError;

use crate::program::on_path;

const INSIDE_THE_APP: &str = "Contents/Helpers/Rex Server.rex/Contents/MacOS/rex";

pub fn rex_program(
    path_variable: Option<&OsStr>,
    applications: &Path,
    app_name: &str,
) -> Result<PathBuf, JumpError> {
    if let Some(found) = on_path("rex", path_variable) {
        return Ok(found);
    }
    let bundled = applications
        .join(format!("{app_name}.app"))
        .join(INSIDE_THE_APP);
    if bundled.is_file() {
        return Ok(bundled);
    }
    Err(JumpError::Unreachable(format!(
        "rex is not on PATH and not at {}",
        bundled.display()
    )))
}

#[cfg(test)]
mod tests;
