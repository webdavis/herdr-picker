use std::env;
use std::fmt;
use std::path::Path;
use std::time::Duration;

use herdr_picker_application::JumpError;
use herdr_picker_domain::NewPane;
use herdr_picker_protocol::{
    self as protocol, Handover, PaneInfo, PaneKind, Refusal, Underneath, guard, handover,
    parse_instruction,
};

use crate::herdr_socket::HerdrSocket;
use crate::program::on_path;
use crate::response::jump_error;

mod calls;

pub use calls::{Sent, drain_queue, send_prompt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandoverError {
    Unreadable(String),
    Refused(Refusal),
    Herdr(JumpError),
}

impl fmt::Display for HandoverError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable(line) => write!(formatter, "the picker answered {line:?}"),
            Self::Refused(refusal) => write!(formatter, "{refusal}"),
            Self::Herdr(failure) => write!(formatter, "{failure}"),
        }
    }
}

impl From<Refusal> for HandoverError {
    fn from(refusal: Refusal) -> Self {
        Self::Refused(refusal)
    }
}

impl From<JumpError> for HandoverError {
    fn from(failure: JumpError) -> Self {
        Self::Herdr(failure)
    }
}

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

    fn screen(&mut self, pane_id: &str) -> Result<String, JumpError> {
        let result = self
            .socket
            .send(|id| protocol::screen_request(id, pane_id))?;
        protocol::screen_of(&result).map_err(jump_error)
    }

    fn type_text(&mut self, pane_id: &str, text: &str, submit: bool) -> Result<(), JumpError> {
        self.socket
            .send(|id| protocol::type_request(id, pane_id, text))?;
        if submit {
            self.socket
                .send(|id| protocol::enter_request(id, pane_id))?;
        }
        Ok(())
    }

    fn open(
        &mut self,
        beside: &PaneInfo,
        pane: NewPane,
        folder: &str,
    ) -> Result<String, JumpError> {
        let result = self
            .socket
            .send(|id| protocol::open_request(id, beside, pane, folder))?;
        let opened = protocol::pane_of(&result).map_err(jump_error)?.pane_id;
        if matches!(pane, NewPane::Left | NewPane::Above) {
            self.socket
                .send(|id| protocol::swap_request(id, &opened, &beside.pane_id))?;
        }
        Ok(opened)
    }
}

// A picker's result, handed to the pane it was opened over the way what runs there takes it.
pub fn hand_over(
    panes: &mut SocketPanes,
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
        Handover::Herdr(call) => calls::carry_out(panes, queue, &call),
        Handover::Type { text, submit } => {
            if matches!(kind, PaneKind::Claude | PaneKind::Codex) {
                guard(&kind, &pane.status, &panes.screen(target)?)?;
            }
            Ok(panes.type_text(target, &text, submit)?)
        }
        Handover::Open {
            pane: place,
            folder,
            run,
        } => {
            let opened = panes.open(&pane, place, folder.as_deref().unwrap_or(&pane.cwd))?;
            match run {
                Some(command) => Ok(panes.type_text(&opened, &command, true)?),
                None => Ok(()),
            }
        }
    }
}

#[cfg(test)]
mod tests;
