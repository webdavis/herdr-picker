use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, ExitStatus};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use pwe_domain::PickerRow;
use pwe_protocol::{RowError, parse_header, parse_row};
use rustix::process::kill_process_group;

use super::row_sink::RowSink;
use super::{PickerRunError, signal};
use crate::private::write_private_atomically;

pub(crate) struct Feed<S> {
    pub(crate) source: Child,
    pub(crate) sink: S,
    pub(crate) error_file: PathBuf,
    pub(crate) said_file: PathBuf,
    pub(crate) stopping: Arc<AtomicBool>,
}

pub(crate) fn feed<S: RowSink>(feed: Feed<S>) -> Result<Vec<PickerRow>, PickerRunError> {
    let Feed {
        mut source,
        mut sink,
        error_file,
        said_file,
        stopping,
    } = feed;
    let rows_out = source.stdout.take().expect("the picker's stdout is piped");
    let bad_row = deliver(rows_out, &mut sink);
    if bad_row.is_some() {
        signal(source.id(), kill_process_group);
    }
    let status = source.wait();
    let said = fs::read_to_string(said_file).unwrap_or_default();
    let refusal = bad_row
        .map(PickerRunError::BadRow)
        .or_else(|| failure(status, &stopping, &said));
    if let Some(refusal) = &refusal {
        let _ = write_private_atomically(&error_file, refusal.to_string().as_bytes());
    }
    let rows = sink.finish();
    refusal.map_or(Ok(rows), Err)
}

fn deliver(rows_out: impl io::Read, sink: &mut impl RowSink) -> Option<RowError> {
    let mut taken = 0;
    for (index, line) in BufReader::new(rows_out).lines().enumerate() {
        if let Some(header) = line.as_deref().ok().and_then(parse_header) {
            sink.header(&header);
            continue;
        }
        let (row, line) = match read_row(line, index + 1) {
            Ok(read) => read,
            Err(bad_row) => return Some(bad_row),
        };
        if sink.take(taken, row, &line).is_err() {
            break;
        }
        taken += 1;
    }
    None
}

fn read_row(line: io::Result<String>, number: usize) -> Result<(PickerRow, String), RowError> {
    let line = line.map_err(|failure| RowError {
        line: number,
        reason: failure.to_string(),
    })?;
    Ok((parse_row(&line, number)?, line))
}

pub(crate) fn failure(
    status: io::Result<ExitStatus>,
    stopping: &AtomicBool,
    said: &str,
) -> Option<PickerRunError> {
    let reason = match status {
        Ok(status) if status.success() || stopping.load(Ordering::SeqCst) => return None,
        Ok(status) => status.to_string(),
        Err(failure) => failure.to_string(),
    };
    Some(PickerRunError::CommandFailed(match said.trim() {
        "" => reason,
        said => format!("{reason}: {said}"),
    }))
}

#[cfg(test)]
mod tests;
