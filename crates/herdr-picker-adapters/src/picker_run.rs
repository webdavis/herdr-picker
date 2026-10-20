use std::fs::{File, create_dir_all};
use std::io;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use herdr_picker_domain::PickerRow;
use rustix::process::{Pid, Signal, kill_process_group};

use crate::picker_files::picker_error_file;

mod error;
mod feed;

pub use error::PickerRunError;

const RESIGNAL_EVERY: Duration = Duration::from_millis(20);

pub fn run_picker(
    command: &Path,
    finder: &Path,
    finder_arguments: &[String],
    cache: &Path,
) -> Result<Option<PickerRow>, PickerRunError> {
    // Its own process group, so stopping it also stops whatever it spawned to print rows.
    let mut source = spawn(
        Command::new(command)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .process_group(0),
        command,
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
        thread::spawn(move || {
            feed::feed(feed::Feed {
                source,
                rows_in,
                cache: cache_file,
                error_file,
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
    Ok(chosen(&picked.stdout, rows))
}

fn start_finder(
    finder: &Path,
    arguments: &[String],
    cache: &Path,
) -> Result<(Child, File), PickerRunError> {
    let cache_file = create_cache(cache).map_err(|failure| PickerRunError::CannotCache {
        path: cache.to_owned(),
        reason: failure.to_string(),
    })?;
    let finder_process = spawn(
        Command::new(finder)
            .args(arguments)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped()),
        finder,
    )?;
    Ok((finder_process, cache_file))
}

fn create_cache(cache: &Path) -> io::Result<File> {
    if let Some(parent) = cache.parent() {
        create_dir_all(parent)?;
    }
    File::create(cache)
}

fn spawn(command: &mut Command, program: &Path) -> Result<Child, PickerRunError> {
    command
        .spawn()
        .map_err(|failure| PickerRunError::CannotStart {
            program: program.to_owned(),
            reason: failure.to_string(),
        })
}

fn chosen(picked: &[u8], rows: Vec<PickerRow>) -> Option<PickerRow> {
    let picked = String::from_utf8_lossy(picked);
    let index: usize = picked.split('\t').next()?.trim().parse().ok()?;
    rows.into_iter().nth(index)
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
