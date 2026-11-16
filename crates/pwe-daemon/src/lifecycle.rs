use std::fmt;
use std::fs::{DirBuilder, File, OpenOptions};
use std::io;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use pwe_adapters::{DaemonAskError, ask_daemon};
use pwe_protocol::{DaemonRequest, parse_daemon_status};

const ANSWER_DEADLINE: Duration = Duration::from_secs(5);
const POLL_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Debug, PartialEq, Eq)]
pub enum Started {
    Now(u32),
    AlreadyRunning(u32),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Stopped {
    Now,
    WasNotRunning,
}

#[derive(Debug)]
pub enum LifecycleError {
    Unreachable(DaemonAskError),
    Launch(io::Error),
    EndedAtOnce(PathBuf),
    Silent(Duration),
}

impl fmt::Display for LifecycleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreachable(error) => write!(formatter, "{error}"),
            Self::Launch(error) => write!(formatter, "could not launch the pwe daemon: {error}"),
            Self::EndedAtOnce(log) => write!(
                formatter,
                "the pwe daemon ended as it started; its log is {}",
                log.display()
            ),
            Self::Silent(deadline) => write!(
                formatter,
                "the pwe daemon did not answer within {} seconds",
                deadline.as_secs()
            ),
        }
    }
}

fn running_pid(socket: &Path) -> Result<Option<u32>, LifecycleError> {
    match ask_daemon(socket, DaemonRequest::Status) {
        Ok(result) => parse_daemon_status(&result)
            .map(|status| Some(status.pid))
            .map_err(|refusal| LifecycleError::Unreachable(DaemonAskError::Refused(refusal))),
        Err(DaemonAskError::NotRunning) => Ok(None),
        Err(error) => Err(LifecycleError::Unreachable(error)),
    }
}

pub fn start_daemon(program: &Path, socket: &Path, log: &Path) -> Result<Started, LifecycleError> {
    if let Some(pid) = running_pid(socket)? {
        return Ok(Started::AlreadyRunning(pid));
    }
    let mut daemon = launch(program, log).map_err(LifecycleError::Launch)?;
    let pid = wait_for_answer(socket, log, || matches!(daemon.try_wait(), Ok(Some(_))))?;
    Ok(Started::Now(pid))
}

pub(crate) fn wait_for_answer(
    socket: &Path,
    log: &Path,
    mut has_ended: impl FnMut() -> bool,
) -> Result<u32, LifecycleError> {
    let deadline = Instant::now() + ANSWER_DEADLINE;
    while Instant::now() < deadline {
        if let Some(pid) = running_pid(socket)? {
            return Ok(pid);
        }
        if has_ended() {
            return Err(LifecycleError::EndedAtOnce(log.to_path_buf()));
        }
        thread::sleep(POLL_INTERVAL);
    }
    Err(LifecycleError::Silent(ANSWER_DEADLINE))
}

fn launch(program: &Path, log: &Path) -> io::Result<Child> {
    Command::new(program)
        .args(["daemon", "run"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(open_log(log)?)
        .process_group(0)
        .spawn()
}

fn open_log(log: &Path) -> io::Result<File> {
    if let Some(folder) = log.parent() {
        DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(folder)?;
    }
    OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .open(log)
}

pub fn stop_daemon(socket: &Path) -> Result<Stopped, LifecycleError> {
    match ask_daemon(socket, DaemonRequest::Stop) {
        Err(DaemonAskError::NotRunning) => return Ok(Stopped::WasNotRunning),
        Err(error) => return Err(LifecycleError::Unreachable(error)),
        Ok(_) => {}
    }
    let deadline = Instant::now() + ANSWER_DEADLINE;
    while Instant::now() < deadline {
        if running_pid(socket)?.is_none() {
            return Ok(Stopped::Now);
        }
        thread::sleep(POLL_INTERVAL);
    }
    Err(LifecycleError::Silent(ANSWER_DEADLINE))
}
