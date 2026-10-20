use std::fs::File;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use herdr_picker_domain::PickerRow;
use rustix::process::{Pid, Signal, kill_process_group};

use crate::picker_files::{cached_rows, picker_error_file, picker_header_file, picker_said_file};
use crate::private::create_private;

mod error;
pub(crate) mod feed;

pub use error::PickerRunError;

// What the finder answered: its query, the key that closed it, and the rows it marked.
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
    let said = create_private(&said_file).map_err(|failure| PickerRunError::CannotCache {
        path: said_file.clone(),
        reason: failure.to_string(),
    })?;
    // Its own process group, so stopping it also stops whatever it spawned to print rows.
    let mut source = spawn(
        command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(said)
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
    // A group signal misses a child forked at that instant, so it repeats until the rows end.
    while !feeder.is_finished() {
        signal(source_pid, kill_process_group);
        thread::sleep(RESIGNAL_EVERY);
    }
    let rows = feeder.join().expect("feeding rows does not panic")?;
    let picked = picked.map_err(|failure| PickerRunError::FinderFailed(failure.to_string()))?;
    // A live step lists its rows again as the user types, so the cache holds the last list.
    Ok(chosen(&picked.stdout, cached_rows(cache).unwrap_or(rows)))
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

// The finder prints the query, then the key, then one index-tab-label line per marked row.
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
