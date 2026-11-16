mod lines;

use std::process::Child;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, SyncSender, TryRecvError};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use pwe_adapters::RexEventStream;
use pwe_domain::Backoff;
use pwe_protocol::RexLink;

use crate::arrival::{Arrival, RexNews};
use lines::{Ended, pass_on_events};

type RunningChild = Arc<Mutex<Option<Child>>>;

pub(crate) struct RexFollower {
    stop_asked: Sender<()>,
    child: RunningChild,
    following: JoinHandle<()>,
}

impl RexFollower {
    pub(crate) fn start(stream: RexEventStream, arrivals: SyncSender<Arrival>) -> Self {
        let (stop_asked, stop_heard) = mpsc::channel();
        let child = RunningChild::default();
        let shared = Arc::clone(&child);
        let following = thread::spawn(move || follow(&stream, &arrivals, &stop_heard, &shared));
        Self {
            stop_asked,
            child,
            following,
        }
    }

    pub(crate) fn stop(self) {
        drop(self.stop_asked);
        if let Some(child) = held(&self.child).as_mut() {
            let _ = child.kill();
        }
        let _ = self.following.join();
    }
}

fn held(child: &RunningChild) -> MutexGuard<'_, Option<Child>> {
    child
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn follow(
    stream: &RexEventStream,
    arrivals: &SyncSender<Arrival>,
    stop_heard: &Receiver<()>,
    child: &RunningChild,
) {
    let mut backoff = Backoff::default();
    loop {
        if let Ok((running, output)) = stream.open() {
            *held(child) = Some(running);
            let ended = match stop_heard.try_recv() {
                Err(TryRecvError::Empty) => pass_on_events(output, arrivals, &mut backoff),
                _ => Ended::Stopping,
            };
            if let Some(mut finished) = held(child).take() {
                let _ = finished.kill();
                let _ = finished.wait();
            }
            if ended == Ended::Stopping || !told(arrivals, RexLink::Reconnecting) {
                return;
            }
        }
        let wait = Duration::from_secs(backoff.wait_after_failure());
        if stop_heard.recv_timeout(wait) != Err(RecvTimeoutError::Timeout) {
            return;
        }
    }
}

fn told(arrivals: &SyncSender<Arrival>, link: RexLink) -> bool {
    arrivals.send(Arrival::Rex(RexNews::Linked(link))).is_ok()
}

#[cfg(test)]
mod tests;
