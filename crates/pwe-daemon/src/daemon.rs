use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender};
use std::thread;
use std::time::Duration;

use pwe_adapters::RexEventStream;
use pwe_protocol::{DaemonStatus, Listens, RexLink};

use crate::arrival::{Arrival, Request, WAITING_ARRIVALS};

use crate::clock::seconds_since_1970;
use crate::core::DaemonCore;
use crate::daemon_start_error::DaemonStartError;
use crate::rex_stream::RexFollower;
use crate::socket_claim::{ClaimRefusal, SocketClaim};

const REQUEST_DEADLINE: Duration = Duration::from_secs(2);
const LONGEST_REQUEST_BYTES: u64 = 64 * 1024;

pub struct Daemon {
    claim: SocketClaim,
    core: DaemonCore,
    rex: Option<RexEventStream>,
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
            rex: None,
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
        self.rex = rex_program.map(RexEventStream::at);
    }

    pub fn serve(self) -> io::Result<()> {
        let listener = self.claim.listener.try_clone()?;
        let (arrivals, received) = mpsc::sync_channel(WAITING_ARRIVALS);
        let follower = self
            .rex
            .map(|stream| RexFollower::start(stream, arrivals.clone()));
        let listening = thread::spawn(move || listen(&listener, &arrivals));
        let claim = self.claim;
        answer_until_stopped(self.core, &received, || claim.remove_socket());
        drop(received);
        if let Some(follower) = follower {
            follower.stop();
        }
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
                core.hear_rex(news);
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

fn listen(listener: &UnixListener, requests: &SyncSender<Arrival>) {
    for mut stream in listener.incoming().flatten() {
        if let Ok(false) = exchange(&mut stream, requests) {
            return;
        }
    }
}

fn exchange(stream: &mut UnixStream, requests: &SyncSender<Arrival>) -> io::Result<bool> {
    stream.set_read_timeout(Some(REQUEST_DEADLINE))?;
    stream.set_write_timeout(Some(REQUEST_DEADLINE))?;
    let mut line = String::new();
    BufReader::new(Read::by_ref(stream).take(LONGEST_REQUEST_BYTES)).read_line(&mut line)?;
    let (answer_to, answered) = mpsc::channel();
    requests
        .send(Arrival::Request(Request { line, answer_to }))
        .map_err(|_| io::Error::other("the daemon stopped answering"))?;
    let answer = answered
        .recv()
        .map_err(|_| io::Error::other("the daemon stopped answering"))?;
    stream.write_all(answer.reply_line.as_bytes())?;
    Ok(answer.keeps_serving)
}

#[cfg(test)]
mod tests;
