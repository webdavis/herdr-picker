use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

use rustix::process::{Pid, Signal, kill_process_group};

use super::PickerRunError;
use crate::picker_files::picker_said_file;
use crate::private::create_private;

const RESIGNAL_EVERY: Duration = Duration::from_millis(20);

pub(crate) fn start_rows(
    command: &mut Command,
    cache: &Path,
) -> Result<(Child, PathBuf), PickerRunError> {
    let said_file = picker_said_file(cache);
    let stderr_in_a_file_that_cannot_block =
        create_private(&said_file).map_err(|failure| PickerRunError::CannotCache {
            path: said_file.clone(),
            reason: failure.to_string(),
        })?;
    let source = spawn(
        command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(stderr_in_a_file_that_cannot_block)
            .process_group(0),
    )?;
    Ok((source, said_file))
}

pub(crate) fn stop_the_group_again_until_feeding_ends_to_catch_late_forks<T>(
    source_pid: u32,
    feeder: &thread::JoinHandle<T>,
) {
    while !feeder.is_finished() {
        signal(source_pid, kill_process_group);
        thread::sleep(RESIGNAL_EVERY);
    }
}

pub(super) fn spawn(command: &mut Command) -> Result<Child, PickerRunError> {
    command
        .spawn()
        .map_err(|failure| PickerRunError::CannotStart {
            program: PathBuf::from(command.get_program()),
            reason: failure.to_string(),
        })
}

pub(super) fn stop(source: &mut Child) {
    signal(source.id(), kill_process_group);
    let _ = source.wait();
}

pub(crate) fn signal(pid: u32, send: fn(Pid, Signal) -> rustix::io::Result<()>) {
    if let Some(pid) = i32::try_from(pid).ok().and_then(Pid::from_raw) {
        let _ = send(pid, Signal::TERM);
    }
}
