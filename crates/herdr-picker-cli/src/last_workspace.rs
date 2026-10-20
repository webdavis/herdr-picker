use std::path::Path;

use herdr_picker_adapters::{CliWorkspaceDirectory, DEADLINE, SocketWorkspaceDirectory};
use herdr_picker_application::{JumpError, WorkspaceHistory, bounce};
use herdr_picker_domain::Bounce;

pub(crate) fn run(
    socket_path: Option<String>,
    herdr_binary: Option<String>,
    history: &mut impl WorkspaceHistory,
) -> Result<Bounce, JumpError> {
    let target_before_any_reply = history.read();
    let over_socket = match socket_path.filter(|path| !path.is_empty()) {
        Some(path) => {
            let mut directory = SocketWorkspaceDirectory::at(Path::new(&path), DEADLINE);
            bounce(&mut directory, history, &target_before_any_reply)
        }
        None => Err(JumpError::Unreachable("HERDR_SOCKET_PATH is unset".into())),
    };
    let socket_failure = match over_socket {
        Ok(outcome) => return Ok(outcome),
        Err(failure) => failure,
    };
    let mut directory = CliWorkspaceDirectory::using(herdr_binary);
    bounce(&mut directory, history, &target_before_any_reply).map_err(|cli_failure| {
        JumpError::BothPathsFailed {
            cli: cli_failure.to_string(),
            socket: socket_failure.to_string(),
        }
    })
}

#[cfg(test)]
mod tests;
