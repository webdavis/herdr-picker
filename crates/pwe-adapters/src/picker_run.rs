use std::fs::File;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use pwe_domain::PickerRow;
use rustix::process::{Pid, Signal, kill_process_group};

use crate::picker_files::{cached_rows, picker_error_file, picker_header_file, picker_said_file};
use crate::private::create_private;

mod error;
pub(crate) mod feed;

pub use error::PickerRunError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picked {
    pub query: String,
    pub key: String,
    pub rows: Vec<PickerRow>,
}

const RESIGNAL_EVERY: Duration = Duration::from_millis(20);

pub fn run_picker(
    command: &mut Command,
    finder: &Path,
    finder_arguments: &[String],
    cache: &Path,
) -> Result<Option<Picked>, PickerRunError> {
    let said_file = picker_said_file(cache);
    let stderr_in_a_file_that_cannot_block =
        create_private(&said_file).map_err(|failure| PickerRunError::CannotCache {
            path: said_file.clone(),
            reason: failure.to_string(),
        })?;
    let mut source = spawn(
        command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(stderr_in_a_file_that_cannot_block)
            .process_group(0),
    )?;
    let (mut finder_process, cache_file) = match start_finder(finder, finder_arguments, cache) {
        Ok(started) => started,
        Err(refusal) => {
            stop(&mut source);
            return Err(refusal);
        }
    };
    let rows_in = finder_process
        .stdin
        .take()
        .expect("the finder's stdin is piped");
    let source_pid = source.id();
    let stopping = Arc::new(AtomicBool::new(false));
    let feeder = {
        let (stopping, error_file) = (Arc::clone(&stopping), picker_error_file(cache));
        let header_file = picker_header_file(cache);
        thread::spawn(move || {
            feed::feed(feed::Feed {
                source,
                rows_in,
                cache: cache_file,
                error_file,
                header_file,
                said_file,
                stopping,
            })
        })
    };
    let picked = finder_process.wait_with_output();
    stopping.store(true, Ordering::SeqCst);
    stop_the_group_again_until_feeding_ends_to_catch_late_forks(source_pid, &feeder);
    let rows = feeder.join().expect("feeding rows does not panic")?;
    let picked = picked.map_err(|failure| PickerRunError::FinderFailed(failure.to_string()))?;
    Ok(chosen(&picked.stdout, cached_rows(cache).unwrap_or(rows)))
}

fn stop_the_group_again_until_feeding_ends_to_catch_late_forks<T>(
    source_pid: u32,
    feeder: &thread::JoinHandle<T>,
) {
    while !feeder.is_finished() {
        signal(source_pid, kill_process_group);
        thread::sleep(RESIGNAL_EVERY);
    }
}

fn start_finder(
    finder: &Path,
    arguments: &[String],
    cache: &Path,
) -> Result<(Child, File), PickerRunError> {
    let cache_file = create_private(cache).map_err(|failure| PickerRunError::CannotCache {
        path: cache.to_owned(),
        reason: failure.to_string(),
    })?;
    let finder_process = spawn(
        Command::new(finder)
            .args(arguments)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped()),
    )?;
    Ok((finder_process, cache_file))
}

fn spawn(command: &mut Command) -> Result<Child, PickerRunError> {
    command
        .spawn()
        .map_err(|failure| PickerRunError::CannotStart {
            program: PathBuf::from(command.get_program()),
            reason: failure.to_string(),
        })
}

fn chosen(picked: &[u8], rows: Vec<PickerRow>) -> Option<Picked> {
    let picked = String::from_utf8_lossy(picked);
    let mut lines = picked.lines();
    let (query, key) = (lines.next()?.to_string(), lines.next()?.to_string());
    let rows = lines
        .filter_map(|line| line.split('\t').next()?.trim().parse::<usize>().ok())
        .filter_map(|index| rows.get(index).cloned())
        .collect();
    Some(Picked { query, key, rows })
}

fn stop(source: &mut Child) {
    signal(source.id(), kill_process_group);
    let _ = source.wait();
}

fn signal(pid: u32, send: fn(Pid, Signal) -> rustix::io::Result<()>) {
    if let Some(pid) = i32::try_from(pid).ok().and_then(Pid::from_raw) {
        let _ = send(pid, Signal::TERM);
    }
}

#[cfg(test)]
mod tests;
