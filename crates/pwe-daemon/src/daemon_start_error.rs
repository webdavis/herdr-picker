use std::fmt;
use std::io;
use std::path::PathBuf;

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
