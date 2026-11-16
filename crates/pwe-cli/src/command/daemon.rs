use std::env;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use pwe_adapters::{daemon_log_file, daemon_socket_file, system_job_file};
use pwe_daemon::{Daemon, Started, Stopped, daemon_report, start_daemon, stop_daemon};

use super::{CommandError, failed};

pub(super) fn execute(words: &[&str]) -> Option<Result<(), CommandError>> {
    Some(match words {
        ["daemon", "run"] => run(),
        ["daemon", "start"] => start(),
        ["daemon", "stop"] => stop(),
        ["daemon", "status"] => status(false),
        ["daemon", "status", "--output", "json"] => status(true),
        _ => return None,
    })
}

fn variable(name: &str) -> Option<String> {
    env::var(name).ok()
}

fn socket() -> PathBuf {
    daemon_socket_file(
        variable("XDG_RUNTIME_DIR").as_deref(),
        variable("XDG_STATE_HOME").as_deref(),
        variable("HOME").as_deref(),
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

fn start() -> Result<(), CommandError> {
    let program = env::current_exe().map_err(failed)?;
    let log = daemon_log_file(
        variable("XDG_STATE_HOME").as_deref(),
        variable("HOME").as_deref(),
    );
    match start_daemon(&program, &socket(), &log).map_err(failed)? {
        Started::Now(pid) => println!("The pwe daemon started as process {pid}."),
        Started::AlreadyRunning(pid) => {
            println!("The pwe daemon is already running as process {pid}.")
        }
    }
    Ok(())
}

fn stop() -> Result<(), CommandError> {
    match stop_daemon(&socket()).map_err(failed)? {
        Stopped::Now => println!("The pwe daemon stopped."),
        Stopped::WasNotRunning => println!("The pwe daemon was not running."),
    }
    Ok(())
}

fn status(as_json: bool) -> Result<(), CommandError> {
    let enabled = system_job_file(
        variable("XDG_CONFIG_HOME").as_deref(),
        variable("HOME").as_deref(),
    )
    .exists();
    let report = daemon_report(&socket(), enabled).map_err(failed)?;
    if as_json {
        println!("{}", report.json());
    } else {
        print!("{}", report.plain_words());
    }
    Ok(())
}
