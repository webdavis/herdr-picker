use std::env;
use std::path::Path;
use std::time::Duration;

use pwe_application::JumpError;
use pwe_domain::{HerdrCall, NewPane};
use pwe_protocol::{
    self as protocol, Handover, PaneInfo, PaneKind, Underneath, guard, handover, parse_instruction,
};

use crate::herdr_socket::HerdrSocket;
use crate::program::on_path;
use crate::response::jump_error;

mod calls;
mod error;
mod host;

pub use calls::{Sent, drain_queue, send_prompt};
pub use error::HandoverError;
pub use host::PaneHost;

pub struct SocketPanes {
    socket: HerdrSocket,
}

impl SocketPanes {
    pub fn at(path: &Path, deadline: Duration) -> Self {
        Self {
            socket: HerdrSocket::at(path, deadline),
        }
    }

    pub fn notify(&mut self, body: &str) -> Result<(), JumpError> {
        self.socket
            .send(|id| protocol::notify_request(id, body))
            .map(|_| ())
    }

    pub fn pane(&mut self, pane_id: &str) -> Result<PaneInfo, JumpError> {
        let result = self.socket.send(|id| protocol::pane_request(id, pane_id))?;
        protocol::pane_of(&result).map_err(jump_error)
    }
}

impl PaneHost for SocketPanes {
    fn pane(&mut self, target: &str) -> Result<PaneInfo, JumpError> {
        SocketPanes::pane(self, target)
    }

    fn screen(&mut self, target: &str) -> Result<String, JumpError> {
        let result = self
            .socket
            .send(|id| protocol::screen_request(id, target))?;
        protocol::screen_of(&result).map_err(jump_error)
    }

    fn type_text(&mut self, target: &str, text: &str) -> Result<(), JumpError> {
        self.socket
            .send(|id| protocol::type_request(id, target, text))
            .map(|_| ())
    }

    fn press_enter(&mut self, target: &str) -> Result<(), JumpError> {
        self.socket
            .send(|id| protocol::enter_request(id, target))
            .map(|_| ())
    }

    fn open_pane(
        &mut self,
        beside: &PaneInfo,
        place: NewPane,
        folder: &str,
    ) -> Result<String, JumpError> {
        let result = self
            .socket
            .send(|id| protocol::open_request(id, beside, place, folder))?;
        let opened = protocol::pane_of(&result).map_err(jump_error)?.pane_id;
        if matches!(place, NewPane::Left | NewPane::Above) {
            self.socket
                .send(|id| protocol::swap_request(id, &opened, &beside.pane_id))?;
        }
        Ok(opened)
    }

    fn host_call(&mut self, call: &HerdrCall, queue: &Path) -> Result<(), HandoverError> {
        calls::carry_out(self, queue, call)
    }
}

pub fn hand_over(
    panes: &mut impl PaneHost,
    target: &str,
    line: &str,
    editor: &str,
    queue: &Path,
) -> Result<(), HandoverError> {
    let instruction =
        parse_instruction(line).ok_or_else(|| HandoverError::Unreadable(line.to_string()))?;
    let pane = panes.pane(target)?;
    let kind = PaneKind::of(pane.agent.as_deref());
    let path_variable = env::var_os("PATH");
    let underneath = Underneath {
        kind: kind.clone(),
        cwd: &pane.cwd,
        editor,
        is_path: &|word| Path::new(&pane.cwd).join(word).exists(),
        is_program: &|word| on_path(word, path_variable.as_deref()).is_some(),
    };
    match handover(&instruction, &underneath)? {
        Handover::Herdr(call) => panes.host_call(&call, queue),
        Handover::Type { text, submit } => {
            let insert_lands_after_any_shell_draft = !submit && kind == PaneKind::Shell;
            if !insert_lands_after_any_shell_draft {
                guard(&kind, &pane.status, &panes.screen(target)?)?;
            }
            Ok(type_line(panes, target, &text, submit)?)
        }
        Handover::Open {
            pane: place,
            folder,
            run,
        } => {
            let opened = panes.open_pane(&pane, place, folder.as_deref().unwrap_or(&pane.cwd))?;
            match run {
                Some(command) => Ok(type_line(panes, &opened, &command, true)?),
                None => Ok(()),
            }
        }
    }
}

fn type_line(
    panes: &mut impl PaneHost,
    target: &str,
    text: &str,
    submit: bool,
) -> Result<(), JumpError> {
    panes.type_text(target, text)?;
    if submit {
        panes.press_enter(target)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
