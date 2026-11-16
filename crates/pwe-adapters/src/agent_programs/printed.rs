use std::path::Path;
use std::process::{Command, Stdio};

use pwe_protocol::ProtocolError;

pub(super) fn printed_by(program: &Path, arguments: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(arguments)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("{}: {error}", program.display()))?;
    if !output.status.success() {
        return Err(format!(
            "{} {} exited {}: {}",
            program.display(),
            arguments.join(" "),
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub(super) fn unreadable(error: ProtocolError) -> String {
    match error {
        ProtocolError::Malformed(detail) => detail,
        ProtocolError::Server { code, message } => format!("{code}: {message}"),
    }
}
