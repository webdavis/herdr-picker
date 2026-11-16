use std::env;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use pwe_adapters::daemon_socket_file;
use pwe_daemon::Daemon;

use super::{CommandError, failed};

pub(super) fn execute(words: &[&str]) -> Option<Result<(), CommandError>> {
    Some(match words {
        ["daemon", "run"] => run(),
        _ => return None,
    })
}

fn socket() -> PathBuf {
    daemon_socket_file(
        env::var("XDG_RUNTIME_DIR").ok().as_deref(),
        env::var("XDG_STATE_HOME").ok().as_deref(),
        env::var("HOME").ok().as_deref(),
    )
}

fn seconds_since_1970() -> Result<u64, CommandError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .map_err(failed)
}

fn run() -> Result<(), CommandError> {
    Daemon::claim(&socket(), seconds_since_1970()?)
        .map_err(failed)?
        .serve()
        .map_err(failed)
}
