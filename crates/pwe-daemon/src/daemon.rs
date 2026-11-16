use std::fmt;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use pwe_protocol::{DaemonStatus, Listens};

use crate::core::{Answer, DaemonCore};
use crate::socket_claim::{ClaimRefusal, SocketClaim};

const REQUEST_DEADLINE: Duration = Duration::from_secs(2);
const LONGEST_REQUEST_BYTES: u64 = 64 * 1024;

pub struct Daemon {
    claim: SocketClaim,
    core: DaemonCore,
}

#[derive(Debug)]
pub enum DaemonStartError {
    AlreadyRunning(PathBuf),
    Io(io::Error),
}

impl fmt::Display for DaemonStartError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyRunning(socket) => write!(
                formatter,
                "a pwe daemon is already running on {}",
                socket.display()
            ),
            Self::Io(error) => write!(formatter, "the pwe daemon could not start: {error}"),
        }
    }
}

struct Request {
    line: String,
    answer_to: Sender<Answer>,
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
        });
        Ok(Self { claim, core })
    }

    pub fn serve(self) -> io::Result<()> {
        let listener = self.claim.listener.try_clone()?;
        let (requests, received) = mpsc::channel();
        let listening = thread::spawn(move || listen(&listener, &requests));
        answer_until_stopped(&self.core, &received, || self.claim.remove_socket());
        listening
            .join()
            .map_err(|_| io::Error::other("the socket listener panicked"))
    }
}

fn answer_until_stopped(
    core: &DaemonCore,
    received: &Receiver<Request>,
    close_to_new_requests: impl FnOnce(),
) {
    while let Ok(request) = received.recv() {
        let answer = core.answer(&request.line);
        if !answer.keeps_serving {
            close_to_new_requests();
            let _ = request.answer_to.send(answer);
            return;
        }
        let _ = request.answer_to.send(answer);
    }
}

fn listen(listener: &UnixListener, requests: &Sender<Request>) {
    for mut stream in listener.incoming().flatten() {
        if let Ok(false) = exchange(&mut stream, requests) {
            return;
        }
    }
}

fn exchange(stream: &mut UnixStream, requests: &Sender<Request>) -> io::Result<bool> {
    stream.set_read_timeout(Some(REQUEST_DEADLINE))?;
    stream.set_write_timeout(Some(REQUEST_DEADLINE))?;
    let mut line = String::new();
    BufReader::new(Read::by_ref(stream).take(LONGEST_REQUEST_BYTES)).read_line(&mut line)?;
    let (answer_to, answered) = mpsc::channel();
    requests
        .send(Request { line, answer_to })
        .map_err(|_| io::Error::other("the daemon stopped answering"))?;
    let answer = answered
        .recv()
        .map_err(|_| io::Error::other("the daemon stopped answering"))?;
    stream.write_all(answer.reply_line.as_bytes())?;
    Ok(answer.keeps_serving)
}

#[cfg(test)]
mod tests;
