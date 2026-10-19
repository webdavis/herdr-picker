use std::path::Path;
use std::time::Duration;

use herdr_picker_application::{JumpError, PopupHost};
use herdr_picker_domain::Size;
use herdr_picker_protocol as protocol;

use crate::herdr_socket::HerdrSocket;
use crate::response;

pub struct SocketPopupHost {
    socket: HerdrSocket,
    plugin_id: String,
    current_workspace: Option<String>,
}

impl SocketPopupHost {
    pub fn at(
        path: &Path,
        deadline: Duration,
        plugin_id: &str,
        current_workspace: Option<&str>,
    ) -> Self {
        Self {
            socket: HerdrSocket::at(path, deadline),
            plugin_id: plugin_id.to_string(),
            current_workspace: current_workspace.map(str::to_string),
        }
    }
}

impl PopupHost for SocketPopupHost {
    fn window_size(&mut self) -> Result<Size, JumpError> {
        let result = self.socket.send(protocol::layout_request)?;
        protocol::window_of(&result).map_err(response::jump_error)
    }

    fn open(&mut self, size: Size) -> Result<(), JumpError> {
        self.socket.send(|request_id| {
            protocol::picker_request(
                request_id,
                &self.plugin_id,
                size,
                self.current_workspace.as_deref(),
            )
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
