use std::path::Path;
use std::time::Duration;

use herdr_workspace_jump_application::{JumpError, WorkspaceDirectory};
use herdr_workspace_jump_domain::Workspace;
use herdr_workspace_jump_protocol as protocol;

use crate::herdr_socket::HerdrSocket;
use crate::response;

pub struct SocketWorkspaceDirectory {
    socket: HerdrSocket,
}

impl SocketWorkspaceDirectory {
    pub fn at(path: &Path, deadline: Duration) -> Self {
        Self {
            socket: HerdrSocket::at(path, deadline),
        }
    }
}

impl WorkspaceDirectory for SocketWorkspaceDirectory {
    fn list(&mut self) -> Result<Vec<Workspace>, JumpError> {
        let result = self.socket.send(protocol::list_request)?;
        response::workspaces(&result)
    }

    fn focus(&mut self, workspace_id: &str) -> Result<(), JumpError> {
        self.socket
            .send(|request_id| protocol::focus_request(request_id, workspace_id))?;
        Ok(())
    }

    fn create(&mut self, label: &str, cwd: &str) -> Result<(), JumpError> {
        self.socket
            .send(|request_id| protocol::create_request(request_id, label, cwd))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
