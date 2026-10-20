use std::fmt;
use std::fs;
use std::io;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use herdr_picker_protocol::{PaneState, encode_pane_state, parse_pane_state};
use rustix::io::Errno;
use rustix::process::{Pid, test_kill_process};

use crate::private::write_private;

const POLL: Duration = Duration::from_millis(25);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaneWaitError {
    NeverStarted,
    ClosedWithoutResult,
    Failed(String),
    Unreadable(String),
}

impl fmt::Display for PaneWaitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NeverStarted => write!(formatter, "the picker popup did not start"),
            Self::ClosedWithoutResult => {
                write!(formatter, "the picker popup closed without a result")
            }
            Self::Failed(reason) => write!(formatter, "{reason}"),
            Self::Unreadable(reason) => {
                write!(formatter, "cannot read the picker popup's result: {reason}")
            }
        }
    }
}

pub fn write_pane_state(path: &Path, state: &PaneState) -> io::Result<()> {
    write_private(path, encode_pane_state(state).as_bytes())
}

pub fn await_pane_result(
    path: &Path,
    start_within: Duration,
) -> Result<Option<String>, PaneWaitError> {
    let asked = Instant::now();
    loop {
        match read_state(path)? {
            // The popup may write its result and exit between two polls, so read once more.
            Some(PaneState::Running { pid }) if !alive(pid) => return settled(read_state(path)?),
            Some(PaneState::Running { .. }) => {}
            None if asked.elapsed() >= start_within => return Err(PaneWaitError::NeverStarted),
            None => {}
            done => return settled(done),
        }
        thread::sleep(POLL);
    }
}

fn read_state(path: &Path) -> Result<Option<PaneState>, PaneWaitError> {
    match fs::read_to_string(path) {
        Ok(body) => parse_pane_state(&body)
            .map(Some)
            .ok_or_else(|| PaneWaitError::Unreadable(format!("{body:?} is not a picker state"))),
        Err(failure) if failure.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(failure) => Err(PaneWaitError::Unreadable(failure.to_string())),
    }
}

fn settled(state: Option<PaneState>) -> Result<Option<String>, PaneWaitError> {
    match state {
        Some(PaneState::Finished { instruction }) => Ok(instruction),
        Some(PaneState::Failed { reason }) => Err(PaneWaitError::Failed(reason)),
        _ => Err(PaneWaitError::ClosedWithoutResult),
    }
}

pub(crate) fn alive(pid: u32) -> bool {
    let Some(pid) = i32::try_from(pid).ok().and_then(Pid::from_raw) else {
        return false;
    };
    matches!(test_kill_process(pid), Ok(()) | Err(Errno::PERM))
}

#[cfg(test)]
mod tests;
