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
    pane: protocol::PopupPane,
}

impl SocketPopupHost {
    pub fn at(path: &Path, deadline: Duration, plugin_id: &str, pane: protocol::PopupPane) -> Self {
        Self {
            socket: HerdrSocket::at(path, deadline),
            plugin_id: plugin_id.to_string(),
            pane,
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
            protocol::popup_request(request_id, &self.plugin_id, size, &self.pane)
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
