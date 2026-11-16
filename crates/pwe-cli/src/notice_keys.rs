use std::path::Path;

use pwe_adapters::{DaemonAskError, ask_daemon_line};
use pwe_daemon::seconds_since_1970;
use pwe_protocol::{NoticeRequest, notice_request_line};

use crate::command::{CommandError, failed};

const DAEMON_STOPPED: &str = "the pwe daemon is not running; run `pwe daemon start` first";
const SECONDS_IN: [(char, u64); 4] = [('s', 1), ('m', 60), ('h', 3_600), ('d', 86_400)];

pub(crate) fn run(socket: &Path, words: &[&str]) -> Result<(), CommandError> {
    let request = request(words, seconds_since_1970())?;
    println!("{}", ask(socket, &request)?);
    Ok(())
}

pub(crate) fn ask(socket: &Path, request: &NoticeRequest) -> Result<String, CommandError> {
    match ask_daemon_line(socket, &notice_request_line(request)) {
        Ok(result) => Ok(result.to_string()),
        Err(DaemonAskError::NotRunning) => Err(failed(DAEMON_STOPPED)),
        Err(refusal) => Err(failed(refusal)),
    }
}

pub(crate) fn request(words: &[&str], now: u64) -> Result<NoticeRequest, CommandError> {
    let text = |word: &str| word.to_string();
    Ok(match words {
        ["waiting"] => NoticeRequest::Waiting,
        ["hide"] => NoticeRequest::Hide,
        ["jump"] => NoticeRequest::Jump { id: None },
        ["jump", id] => NoticeRequest::Jump { id: Some(text(id)) },
        ["dismiss", id] => NoticeRequest::Dismiss { id: text(id) },
        ["read", id] => NoticeRequest::Read { id: text(id) },
        ["mute", source] => NoticeRequest::Mute {
            source: text(source),
            until: None,
        },
        ["mute", source, "--for", time] => NoticeRequest::Mute {
            source: text(source),
            until: Some(now + seconds_in(time)?),
        },
        ["unmute", source] => NoticeRequest::Unmute {
            source: text(source),
        },
        _ => return Err(CommandError::Usage),
    })
}

fn seconds_in(time: &str) -> Result<u64, CommandError> {
    let unit = time.chars().last().ok_or(CommandError::Usage)?;
    let (_, seconds) = SECONDS_IN
        .iter()
        .find(|(named, _)| *named == unit)
        .ok_or(CommandError::Usage)?;
    let count: u64 = time[..time.len() - 1]
        .parse()
        .map_err(|_| CommandError::Usage)?;
    Ok(count * seconds)
}

#[cfg(test)]
mod tests;
