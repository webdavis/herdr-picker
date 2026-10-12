use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde_json::Value;

use herdr_workspace_jump_application::JumpError;
use herdr_workspace_jump_protocol as protocol;

use crate::response;

pub const DEADLINE: Duration = Duration::from_secs(5);

pub(crate) struct HerdrSocket {
    path: PathBuf,
    deadline: Duration,
    next_request_id: u32,
}

impl HerdrSocket {
    pub(crate) fn at(path: &Path, deadline: Duration) -> Self {
        Self {
            path: path.to_owned(),
            deadline,
            next_request_id: 1,
        }
    }

    pub(crate) fn send(&mut self, encode: impl FnOnce(&str) -> String) -> Result<Value, JumpError> {
        let request = encode(&self.take_request_id());
        let mut stream = UnixStream::connect(&self.path)
            .map_err(|error| JumpError::Unreachable(format!("{}: {error}", self.path.display())))?;
        let expires = Instant::now() + self.deadline;
        write_request(&mut stream, request.as_bytes(), expires)?;
        let response = read_response(&mut stream, expires)?;
        protocol::result_of(&response).map_err(response::jump_error)
    }

    fn take_request_id(&mut self) -> String {
        let request_id = self.next_request_id;
        self.next_request_id += 1;
        format!("req_{request_id}")
    }
}

fn remaining(expires: Instant) -> Result<Duration, JumpError> {
    let remaining = expires.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        Err(JumpError::Transport("request deadline expired".to_string()))
    } else {
        Ok(remaining)
    }
}

fn write_request(
    stream: &mut UnixStream,
    mut bytes: &[u8],
    expires: Instant,
) -> Result<(), JumpError> {
    while !bytes.is_empty() {
        stream
            .set_write_timeout(Some(remaining(expires)?))
            .map_err(|error| {
                JumpError::Transport(format!("could not set write deadline: {error}"))
            })?;
        match stream.write(bytes) {
            Ok(0) => return Err(JumpError::Transport("write made no progress".to_string())),
            Ok(written) => bytes = &bytes[written..],
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(error) => return Err(JumpError::Transport(format!("write failed: {error}"))),
        }
    }
    Ok(())
}

fn read_response(stream: &mut UnixStream, expires: Instant) -> Result<String, JumpError> {
    let mut response = Vec::new();
    let mut bytes = [0; 1024];
    loop {
        if let Err(refused_deadline) = stream.set_read_timeout(Some(remaining(expires)?)) {
            read_only_what_is_already_buffered(stream, refused_deadline)?;
        }
        let count = match stream.read(&mut bytes) {
            Ok(0) => {
                return Err(JumpError::Transport(
                    "herdr closed the connection without answering".to_string(),
                ));
            }
            Ok(count) => count,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(JumpError::Transport(format!("read failed: {error}"))),
        };
        if let Some(end) = bytes[..count].iter().position(|byte| *byte == b'\n') {
            response.extend_from_slice(&bytes[..=end]);
            return String::from_utf8(response)
                .map_err(|error| JumpError::Malformed(format!("response is not UTF-8: {error}")));
        }
        response.extend_from_slice(&bytes[..count]);
    }
}

fn read_only_what_is_already_buffered(
    stream: &UnixStream,
    refused_deadline: std::io::Error,
) -> Result<(), JumpError> {
    stream.set_nonblocking(true).map_err(|error| {
        JumpError::Transport(format!(
            "could not set read deadline: {refused_deadline}; could not make buffered read nonblocking: {error}"
        ))
    })
}

#[cfg(test)]
mod tests;
