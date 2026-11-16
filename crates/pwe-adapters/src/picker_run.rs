use std::fs::File;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use pwe_domain::PickerRow;
use pwe_domain::finder::Found;

use crate::fzf::fzf_ending;
use crate::picker_files::{cached_rows, picker_error_file, picker_header_file};
use crate::private::create_private;

mod error;
pub(crate) mod feed;
pub(crate) mod row_sink;
mod source;

pub use error::PickerRunError;
pub(crate) use source::{
    signal, start_rows, stop_the_group_again_until_feeding_ends_to_catch_late_forks,
};
use source::{spawn, stop};

pub fn run_picker(
    command: &mut Command,
    finder: &Path,
    finder_arguments: &[String],
    cache: &Path,
) -> Result<Option<Found<PickerRow>>, PickerRunError> {
    let (mut source, said_file) = start_rows(command, cache)?;
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
                sink: row_sink::FzfRowSink {
                    rows_in,
                    cache: cache_file,
                    header_file,
                    rows: Vec::new(),
                },
                error_file,
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

fn chosen(picked: &[u8], rows: Vec<PickerRow>) -> Option<Found<PickerRow>> {
    let picked = String::from_utf8_lossy(picked);
    let mut lines = picked.lines();
    let (query, ending) = (lines.next()?.to_string(), fzf_ending(lines.next()?)?);
    let rows = lines
        .filter_map(|line| line.split('\t').next()?.trim().parse::<usize>().ok())
        .filter_map(|index| rows.get(index).cloned())
        .collect();
    Some(Found {
        query,
        ending,
        rows,
    })
}

#[cfg(test)]
mod tests;
