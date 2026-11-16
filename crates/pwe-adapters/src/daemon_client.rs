use std::fmt;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use pwe_protocol::{DaemonError, DaemonRequest, daemon_request_line, parse_daemon_reply};
use serde_json::Value;

const REPLY_DEADLINE: Duration = Duration::from_secs(5);
const LONGEST_REPLY_BYTES: u64 = 1024 * 1024;

#[derive(Debug)]
pub enum DaemonAskError {
    NotRunning,
    Refused(DaemonError),
    Unreachable(io::Error),
}

impl fmt::Display for DaemonAskError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotRunning => write!(formatter, "the pwe daemon is not running"),
            Self::Refused(error) => write!(formatter, "the pwe daemon refused: {}", error.message),
            Self::Unreachable(error) => {
                write!(formatter, "could not reach the pwe daemon: {error}")
            }
        }
    }
}

pub fn daemon_socket_file(
    runtime_dir: Option<&str>,
    state_home: Option<&str>,
    home: Option<&str>,
) -> PathBuf {
    let set = |value: Option<&str>| value.filter(|value| !value.is_empty()).map(PathBuf::from);
    set(runtime_dir)
        .or_else(|| set(state_home))
        .unwrap_or_else(|| PathBuf::from(home.unwrap_or_default()).join(".local/state"))
        .join("pwe/daemon.sock")
}

pub fn ask_daemon(socket: &Path, request: DaemonRequest) -> Result<Value, DaemonAskError> {
    let mut stream = UnixStream::connect(socket).map_err(|error| match error.kind() {
        io::ErrorKind::NotFound | io::ErrorKind::ConnectionRefused => DaemonAskError::NotRunning,
        _ => DaemonAskError::Unreachable(error),
    })?;
    let reply = exchange(&mut stream, request).map_err(DaemonAskError::Unreachable)?;
    parse_daemon_reply(&reply).map_err(DaemonAskError::Refused)
}

fn exchange(stream: &mut UnixStream, request: DaemonRequest) -> io::Result<String> {
    stream.set_read_timeout(Some(REPLY_DEADLINE))?;
    stream.set_write_timeout(Some(REPLY_DEADLINE))?;
    stream.write_all(daemon_request_line(request).as_bytes())?;
    let mut reply = String::new();
    BufReader::new(stream.take(LONGEST_REPLY_BYTES)).read_line(&mut reply)?;
    Ok(reply)
}

#[cfg(test)]
mod tests;
