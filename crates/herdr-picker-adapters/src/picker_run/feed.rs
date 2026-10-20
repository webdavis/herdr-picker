use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ExitStatus};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use herdr_picker_domain::PickerRow;
use herdr_picker_protocol::{RowError, parse_row};
use rustix::process::kill_process_group;

use super::{PickerRunError, signal};

pub(super) struct Feed {
    pub(super) source: Child,
    pub(super) rows_in: ChildStdin,
    pub(super) cache: File,
    pub(super) error_file: PathBuf,
    pub(super) stopping: Arc<AtomicBool>,
}

// A refusal leaves the finder open: its error file is written before the finder's input closes,
// so the finder's load event can show it, and the run still ends in that refusal.
pub(super) fn feed(feed: Feed) -> Result<Vec<PickerRow>, PickerRunError> {
    let Feed {
        mut source,
        mut rows_in,
        mut cache,
        error_file,
        stopping,
    } = feed;
    let rows_out = source.stdout.take().expect("the picker's stdout is piped");
    let (rows, bad_row) = deliver(rows_out, &mut rows_in, &mut cache);
    if bad_row.is_some() {
        signal(source.id(), kill_process_group);
    }
    let status = source.wait();
    let refusal = bad_row
        .map(PickerRunError::BadRow)
        .or_else(|| failure(status, &stopping));
    if let Some(refusal) = &refusal {
        let _ = fs::write(&error_file, refusal.to_string());
    }
    drop(rows_in);
    refusal.map_or(Ok(rows), Err)
}

fn deliver(
    rows_out: impl io::Read,
    rows_in: &mut ChildStdin,
    cache: &mut File,
) -> (Vec<PickerRow>, Option<RowError>) {
    let mut rows = Vec::new();
    for (index, line) in BufReader::new(rows_out).lines().enumerate() {
        let (row, line) = match read_row(line, index + 1) {
            Ok(read) => read,
            Err(bad_row) => return (rows, Some(bad_row)),
        };
        // The cache line goes first: the finder may ask for a row's preview as soon as it shows it.
        let delivered = writeln!(cache, "{line}")
            .and_then(|()| writeln!(rows_in, "{}\t{}", rows.len(), row.label));
        if delivered.is_err() {
            break;
        }
        rows.push(row);
    }
    (rows, None)
}

fn read_row(line: io::Result<String>, number: usize) -> Result<(PickerRow, String), RowError> {
    let line = line.map_err(|failure| RowError {
        line: number,
        reason: failure.to_string(),
    })?;
    Ok((parse_row(&line, number)?, line))
}

fn failure(status: io::Result<ExitStatus>, stopping: &AtomicBool) -> Option<PickerRunError> {
    match status {
        Ok(status) if status.success() || stopping.load(Ordering::SeqCst) => None,
        Ok(status) => Some(PickerRunError::CommandFailed(status.to_string())),
        Err(failure) => Some(PickerRunError::CommandFailed(failure.to_string())),
    }
}
