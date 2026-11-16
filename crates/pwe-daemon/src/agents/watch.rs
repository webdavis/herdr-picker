use std::sync::mpsc::SyncSender;

use pwe_application::AgentSources;
use pwe_domain::{AgentBook, AgentChange, AgentSession, PaneStatus, ProgramReport};
use pwe_protocol::{RexEvent, agent_sessions_value, rex_reported_state};
use serde_json::Value;

pub trait AgentSubscriber: Send {
    fn hear(&mut self, change: &AgentChange);
}

pub(crate) struct AgentWatch {
    book: AgentBook,
    reports: Vec<ProgramReport>,
    panes: Vec<PaneStatus>,
    read: bool,
    asks: SyncSender<()>,
    subscribers: Vec<Box<dyn AgentSubscriber>>,
}

impl AgentWatch {
    pub(crate) fn new(asks: SyncSender<()>) -> Self {
        Self {
            book: AgentBook::default(),
            reports: Vec::new(),
            panes: Vec::new(),
            read: false,
            asks,
            subscribers: Vec::new(),
        }
    }

    pub(crate) fn subscribe(&mut self, subscriber: Box<dyn AgentSubscriber>) {
        self.subscribers.push(subscriber);
    }

    pub(crate) fn ask_for_reading(&self) {
        let _ = self.asks.try_send(());
    }

    pub(crate) fn take_reading(&mut self, sources: AgentSources, now: u64) {
        for refusal in &sources.refusals {
            eprintln!("pwe: {refusal}");
        }
        self.reports = sources.reports;
        self.panes = sources.panes;
        self.read = true;
        self.join(now);
    }

    pub(crate) fn hear_event(&mut self, event: &RexEvent, now: u64) {
        match event {
            RexEvent::ProgramStatusChanged { at, status } => {
                let (state, waiting_for) = rex_reported_state(status);
                let same_program =
                    |pane: &&mut PaneStatus| pane.pane.block == at.block && pane.pid == status.pid;
                match self.panes.iter_mut().find(same_program) {
                    Some(pane) => {
                        pane.state = state;
                        pane.waiting_for = waiting_for;
                        self.join(now);
                    }
                    None => self.ask_for_reading(),
                }
            }
            RexEvent::ProgramStatusRemoved { .. }
            | RexEvent::ProcessChanged { .. }
            | RexEvent::BlockClosed { .. }
            | RexEvent::ChildExited { .. } => self.ask_for_reading(),
            RexEvent::TabChanged { .. } | RexEvent::ViewChanged { .. } | RexEvent::Other => {}
        }
    }

    pub(crate) fn sessions(&self) -> &[AgentSession] {
        self.book.sessions()
    }

    pub(crate) fn listing(&self) -> Option<Value> {
        self.read
            .then(|| agent_sessions_value(&self.book.newest_state_first()).ok())
            .flatten()
    }

    fn join(&mut self, now: u64) {
        for change in self.book.update(&self.reports, &self.panes, now) {
            for subscriber in &mut self.subscribers {
                subscriber.hear(&change);
            }
        }
    }
}

#[cfg(test)]
mod tests;
