use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ExitStatus};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use pwe_domain::PickerRow;
use pwe_protocol::{RowError, parse_header, parse_row};
use rustix::process::kill_process_group;

use super::{PickerRunError, signal};
use crate::private::write_private;

pub(super) struct Feed {
    pub(super) source: Child,
    pub(super) rows_in: ChildStdin,
    pub(super) cache: File,
    pub(super) error_file: PathBuf,
    pub(super) header_file: PathBuf,
    pub(super) said_file: PathBuf,
    pub(super) stopping: Arc<AtomicBool>,
}

pub(super) fn feed(feed: Feed) -> Result<Vec<PickerRow>, PickerRunError> {
    let Feed {
        mut source,
        mut rows_in,
        mut cache,
        error_file,
        header_file,
        said_file,
        stopping,
    } = feed;
    let rows_out = source.stdout.take().expect("the picker's stdout is piped");
    let (rows, bad_row) = deliver(rows_out, &mut rows_in, &mut cache, &header_file);
    if bad_row.is_some() {
        signal(source.id(), kill_process_group);
    }
    let status = source.wait();
    let said = fs::read_to_string(said_file).unwrap_or_default();
    let refusal = bad_row
        .map(PickerRunError::BadRow)
        .or_else(|| failure(status, &stopping, &said));
    if let Some(refusal) = &refusal {
        let _ = write_private(&error_file, refusal.to_string().as_bytes());
    }
    drop(rows_in);
    refusal.map_or(Ok(rows), Err)
}

fn deliver(
    rows_out: impl io::Read,
    rows_in: &mut ChildStdin,
    cache: &mut File,
    header_file: &Path,
) -> (Vec<PickerRow>, Option<RowError>) {
    let mut rows = Vec::new();
    for (index, line) in BufReader::new(rows_out).lines().enumerate() {
        if let Some(header) = line.as_deref().ok().and_then(parse_header) {
            let _ = write_private(header_file, header.as_bytes());
            continue;
        }
        let (row, line) = match read_row(line, index + 1) {
            Ok(read) => read,
            Err(bad_row) => return (rows, Some(bad_row)),
        };
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
