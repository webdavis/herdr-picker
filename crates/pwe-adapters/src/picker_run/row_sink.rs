use std::fs::File;
use std::io::{self, Write};
use std::process::ChildStdin;

use pwe_domain::PickerRow;

pub(super) trait RowSink {
    fn take(&mut self, number: usize, row: &PickerRow, line: &str) -> io::Result<()>;
}

pub(super) struct FzfRowSink {
    pub(super) rows_in: ChildStdin,
    pub(super) cache: File,
}

impl RowSink for FzfRowSink {
    fn take(&mut self, number: usize, row: &PickerRow, line: &str) -> io::Result<()> {
        writeln!(self.cache, "{line}")?;
        writeln!(self.rows_in, "{number}\t{}", row.label)
    }
}
