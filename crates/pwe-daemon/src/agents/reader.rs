use std::sync::mpsc::{Receiver, RecvTimeoutError, SyncSender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use pwe_application::AgentSources;

use super::pace::ReadingPace;
use crate::arrival::Arrival;
use crate::clock::seconds_since_1970;

pub(crate) type ReadAgents = Box<dyn FnMut() -> AgentSources + Send>;

pub(crate) struct AgentReader {
    reading: JoinHandle<()>,
}

impl AgentReader {
    pub(crate) fn start(
        read: ReadAgents,
        refresh_seconds: u64,
        asked: Receiver<()>,
        arrivals: SyncSender<Arrival>,
    ) -> Self {
        let pace = ReadingPace::every(refresh_seconds);
        let reading = thread::spawn(move || read_when_asked(read, pace, &asked, &arrivals));
        Self { reading }
    }

    pub(crate) fn stop(self) {
        let _ = self.reading.join();
    }
}

fn read_when_asked(
    mut read: ReadAgents,
    mut pace: ReadingPace,
    asked: &Receiver<()>,
    arrivals: &SyncSender<Arrival>,
) {
    while asked.recv().is_ok() {
        let wait = pace.wait_before_reading(seconds_since_1970());
        if !waited(asked, Duration::from_secs(wait)) {
            return;
        }
        let sources = read();
        pace.read_at(seconds_since_1970());
        if arrivals.send(Arrival::AgentsRead(sources)).is_err() {
            return;
        }
    }
}

fn waited(asked: &Receiver<()>, wait: Duration) -> bool {
    let deadline = Instant::now() + wait;
    loop {
        let left = deadline.saturating_duration_since(Instant::now());
        match asked.recv_timeout(left) {
            Ok(()) => {}
            Err(RecvTimeoutError::Timeout) => return true,
            Err(RecvTimeoutError::Disconnected) => return false,
        }
    }
}

#[cfg(test)]
mod tests;
