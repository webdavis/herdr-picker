use std::fs::{OpenOptions, create_dir_all};
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

use rustix::io::Errno;
use rustix::process::{Pid, test_kill_process};
use std::os::unix::process::CommandExt;

const POLL_INTERVAL: Duration = Duration::from_millis(10);
pub fn spawn_detached(
    program: &Path,
    arguments: &[&str],
    failure_log: Option<&Path>,
) -> io::Result<()> {
    Command::new(program)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(appending_to(failure_log))
        .process_group(0)
        .spawn()
        .map(|_| ())
}
fn appending_to(failure_log: Option<&Path>) -> Stdio {
    let Some(path) = failure_log else {
        return Stdio::null();
    };
    if let Some(parent) = path.parent() {
        let _ = create_dir_all(parent);
    }
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_or_else(|_| Stdio::null(), Stdio::from)
}
pub fn wait_for_process_exit(pid: u32, cap: Duration) {
    let deadline = Instant::now() + cap;
    while Instant::now() < deadline && process_exists(pid) {
        sleep(POLL_INTERVAL);
    }
}

fn process_exists(pid: u32) -> bool {
    let Some(pid) = i32::try_from(pid).ok().and_then(Pid::from_raw) else {
        return false;
    };
    !no_such_process(test_kill_process(pid))
}

fn no_such_process(signal_test: rustix::io::Result<()>) -> bool {
    signal_test == Err(Errno::SRCH)
}

#[cfg(test)]
mod tests;
