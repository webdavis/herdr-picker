use std::io;
use std::path::PathBuf;
use std::process::{Child, ChildStdout, Command, Stdio};

const EVENTS_OF_EVERY_SESSION: [&str; 4] = ["--autostart=false", "events", "--all", "--json"];

pub struct RexEventStream {
    binary: PathBuf,
}

impl RexEventStream {
    pub fn at(binary: PathBuf) -> Self {
        Self { binary }
    }

    pub fn open(&self) -> io::Result<(Child, ChildStdout)> {
        let mut child = Command::new(&self.binary)
            .args(EVENTS_OF_EVERY_SESSION)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        match child.stdout.take() {
            Some(lines) => Ok((child, lines)),
            None => {
                let _ = child.kill();
                let _ = child.wait();
                Err(io::Error::other("rex events gave no output to read"))
            }
        }
    }
}
