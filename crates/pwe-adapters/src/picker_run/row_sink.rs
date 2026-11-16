use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ChildStdin;

use pwe_domain::PickerRow;

use crate::private::write_private_atomically;

pub(crate) trait RowSink {
    fn take(&mut self, number: usize, row: PickerRow, line: &str) -> io::Result<()>;
    fn header(&mut self, header: &str);
    fn finish(self) -> Vec<PickerRow>;
}

pub(super) struct FzfRowSink {
    pub(super) rows_in: ChildStdin,
    pub(super) cache: File,
    pub(super) header_file: PathBuf,
    pub(super) rows: Vec<PickerRow>,
}

impl RowSink for FzfRowSink {
    fn take(&mut self, number: usize, row: PickerRow, line: &str) -> io::Result<()> {
        writeln!(self.cache, "{line}")?;
        writeln!(self.rows_in, "{number}\t{}", row.label)?;
        self.rows.push(row);
        Ok(())
    }

    fn header(&mut self, header: &str) {
        let _ = write_private_atomically(&self.header_file, header.as_bytes());
    }

    fn finish(self) -> Vec<PickerRow> {
        self.rows
    }
}
