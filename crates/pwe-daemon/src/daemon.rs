use std::io;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::Duration;

use pwe_adapters::RexEventStream;
use pwe_application::AgentSources;
use pwe_protocol::{DaemonStatus, Listens, RexLink};

use crate::agents::{AgentSubscriber, AgentWatch};
use crate::arrival::{Arrival, WAITING_ARRIVALS};
use crate::clock::seconds_since_1970;
use crate::companions::{AgentsToRead, Companions};
use crate::core::DaemonCore;
use crate::daemon_start_error::DaemonStartError;
use crate::listener::listen;
use crate::reopening::{ReopenRunner, Reopening};
use crate::socket_claim::{ClaimRefusal, SocketClaim};

pub struct Daemon {
    claim: SocketClaim,
    core: DaemonCore,
    companions: Companions,
}

impl Daemon {
    pub fn claim(socket: &Path, started_at: u64) -> Result<Self, DaemonStartError> {
        let claim = SocketClaim::take(socket).map_err(|refusal| match refusal {
            ClaimRefusal::AlreadyClaimed => DaemonStartError::AlreadyRunning(socket.to_path_buf()),
            ClaimRefusal::Io(error) => DaemonStartError::Io(error),
        })?;
        let core = DaemonCore::new(DaemonStatus {
            running: true,
            pid: process::id(),
            socket: socket.to_string_lossy().into_owned(),
            listens: Listens::ThisMachineOnly,
            started_at,
            rex: RexLink::Absent,
        });
        Ok(Self {
            claim,
            core,
            companions: Companions::default(),
        })
    }

    pub fn every(&mut self, name: &str, seconds: u64, job: impl FnMut(u64) + Send + 'static) {
        self.core
            .every(name, seconds, seconds_since_1970(), Box::new(job));
    }

    pub fn follow_rex(&mut self, rex_program: Option<PathBuf>) {
        let link = rex_program
            .as_ref()
            .map_or(RexLink::Absent, |_| RexLink::Reconnecting);
        self.core.rex_link(link);
        self.companions.rex = rex_program.map(RexEventStream::at);
    }

    pub fn follow_agents(
        &mut self,
        read: impl FnMut() -> AgentSources + Send + 'static,
        refresh_seconds: u64,
        subscribers: Vec<Box<dyn AgentSubscriber>>,
    ) {
        let (asks, asked) = mpsc::sync_channel(1);
        let ping = asks.clone();
        self.every("agents", refresh_seconds, move |_| {
            let _ = ping.try_send(());
        });
        let mut watch = AgentWatch::new(asks);
        for subscriber in subscribers {
            watch.subscribe(subscriber);
        }
        watch.ask_for_reading();
        self.core.watch_agents(watch);
        self.companions.agents = Some(AgentsToRead {
            read: Box::new(read),
            refresh_seconds,
            asked,
        });
    }

    pub fn reopen_after_rex_restarts(&mut self, records: PathBuf, rex_program: PathBuf) {
        let mut runner = ReopenRunner::new(rex_program);
        let reopen = Box::new(move |panes| runner.reopen(panes));
        self.core
            .reopen_after_rex_restarts(Reopening::new(records, reopen));
    }

    pub fn serve(self) -> io::Result<()> {
        let listener = self.claim.listener.try_clone()?;
        let (arrivals, received) = mpsc::sync_channel(WAITING_ARRIVALS);
        let companions = self.companions.start(&arrivals);
        let listening = thread::spawn(move || listen(&listener, &arrivals));
        let claim = self.claim;
        answer_until_stopped(self.core, &received, || claim.remove_socket());
        drop(received);
        companions.stop();
        listening
            .join()
            .map_err(|_| io::Error::other("the socket listener panicked"))
    }
}

fn answer_until_stopped(
    mut core: DaemonCore,
    received: &Receiver<Arrival>,
    close_to_new_requests: impl FnOnce(),
) {
    loop {
        let now = seconds_since_1970();
        core.run_due(now);
        let next = match core.next_at() {
            Some(at) => received.recv_timeout(Duration::from_secs(at.saturating_sub(now))),
            None => received.recv().map_err(|_| RecvTimeoutError::Disconnected),
        };
        let request = match next {
            Ok(Arrival::Request(request)) => request,
            Ok(Arrival::Rex(news)) => {
                core.hear_rex(news, seconds_since_1970());
                continue;
            }
            Ok(Arrival::AgentsRead(sources)) => {
                core.take_agent_reading(sources, seconds_since_1970());
                continue;
            }
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => return,
        };
        let answer = core.answer(&request.line);
        if !answer.keeps_serving {
            close_to_new_requests();
            let _ = request.answer_to.send(answer);
            return;
        }
        let _ = request.answer_to.send(answer);
    }
}

#[cfg(test)]
mod tests;
