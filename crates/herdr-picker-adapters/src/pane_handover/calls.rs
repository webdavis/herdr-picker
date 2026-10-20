use std::path::Path;

use herdr_picker_application::JumpError;
use herdr_picker_domain::{AgentStatus, HerdrCall, Queued, take_next};
use herdr_picker_protocol::{
    self as protocol, Delivery, PaneKind, Refusal, agents_of, call_request, delivery, tabs_of,
};

use super::{HandoverError, SocketPanes};
use crate::agent_store::{AgentSnapshot, lock_queue, read_queue, write_queue};
use crate::response::{jump_error, workspaces};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sent {
    Now,
    Queued(Refusal),
}

impl SocketPanes {
    pub fn snapshot(&mut self) -> Result<AgentSnapshot, JumpError> {
        let agents = self.socket.send(protocol::agent_list_request)?;
        let spaces = self.socket.send(protocol::list_request)?;
        let tabs = self.socket.send(protocol::tab_list_request)?;
        Ok(AgentSnapshot {
            agents: agents_of(&agents).map_err(jump_error)?,
            workspaces: workspaces(&spaces)?,
            tabs: tabs_of(&tabs).map_err(jump_error)?,
        })
    }

    pub fn call(&mut self, call: &HerdrCall) -> Result<(), JumpError> {
        self.socket.send(|id| call_request(id, call)).map(|_| ())
    }

    fn delivery(&mut self, pane_id: &str) -> Result<Delivery, HandoverError> {
        let pane = self.pane(pane_id)?;
        let kind = PaneKind::of(pane.agent.as_deref());
        let screen = match kind {
            PaneKind::Claude | PaneKind::Codex => self.screen(pane_id)?,
            _ => String::new(),
        };
        Ok(delivery(&kind, &pane.status, &screen)?)
    }
}

// A prompt goes to its agent when the agent can take it, and waits in the queue otherwise.
pub fn send_prompt(
    panes: &mut SocketPanes,
    queue: &Path,
    pane_id: &str,
    text: &str,
) -> Result<Sent, HandoverError> {
    let _held = locked(queue)?;
    match panes.delivery(pane_id)? {
        Delivery::Now => {
            panes.call(&prompt(pane_id, text))?;
            Ok(Sent::Now)
        }
        Delivery::WhenFree(waiting) => {
            let mut queued = read_queue(queue);
            queued.push(Queued {
                pane_id: pane_id.to_string(),
                text: text.to_string(),
            });
            write_queue(queue, &queued).map_err(|failure| {
                HandoverError::Unreadable(format!("cannot save the queue: {failure}"))
            })?;
            Ok(Sent::Queued(waiting))
        }
    }
}

// The status hook: an agent that finished its turn takes its oldest queued prompt, once its input
// line is empty; anything else leaves the queue as it was.
pub fn drain_queue(
    panes: &mut SocketPanes,
    queue: &Path,
    pane_id: &str,
    status: AgentStatus,
) -> Result<bool, HandoverError> {
    if !status.settled() {
        return Ok(false);
    }
    let _held = locked(queue)?;
    let (Some(next), rest) = take_next(read_queue(queue), pane_id) else {
        return Ok(false);
    };
    if panes.delivery(pane_id)? != Delivery::Now {
        return Ok(false);
    }
    write_queue(queue, &rest).map_err(|failure| {
        HandoverError::Unreadable(format!("cannot save the queue: {failure}"))
    })?;
    panes.call(&prompt(pane_id, &next.text))?;
    Ok(true)
}

// A picker's herdr call; a prompt the agent cannot take yet waits, and herdr says so.
pub(super) fn carry_out(
    panes: &mut SocketPanes,
    queue: &Path,
    call: &HerdrCall,
) -> Result<(), HandoverError> {
    match call {
        HerdrCall::Prompt { pane_id, text } => match send_prompt(panes, queue, pane_id, text)? {
            Sent::Now => Ok(()),
            Sent::Queued(waiting) => Ok(panes.notify(&queued_note(&waiting))?),
        },
        other => Ok(panes.call(other)?),
    }
}

fn queued_note(waiting: &Refusal) -> String {
    let why = match waiting {
        Refusal::Blocked => "it waits on an answer",
        Refusal::Draft(_) => "its input line holds your typing",
        _ => "its input line could not be read",
    };
    format!("Queued: {why}. It is sent once the agent is free.")
}

fn locked(queue: &Path) -> Result<std::fs::File, HandoverError> {
    lock_queue(queue)
        .map_err(|failure| HandoverError::Unreadable(format!("cannot lock the queue: {failure}")))
}

fn prompt(pane_id: &str, text: &str) -> HerdrCall {
    HerdrCall::Prompt {
        pane_id: pane_id.to_string(),
        text: text.to_string(),
    }
}

#[cfg(test)]
mod tests;
