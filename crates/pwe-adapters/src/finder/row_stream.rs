use std::io;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};

use pwe_domain::PickerRow;
use pwe_domain::finder::Input;

use crate::picker_files::picker_error_file;
use crate::picker_run::feed::{Feed, feed};
use crate::picker_run::row_sink::RowSink;
use crate::picker_run::{
    PickerRunError, start_rows, stop_the_group_again_until_feeding_ends_to_catch_late_forks,
};

pub struct RowStream {
    source_pid: u32,
    stopping: Arc<AtomicBool>,
    feeder: Option<JoinHandle<()>>,
}

impl RowStream {
    pub fn start(
        command: &mut Command,
        cache: &Path,
        inputs: Sender<Input>,
    ) -> Result<Self, PickerRunError> {
        let (source, said_file) = start_rows(command, cache)?;
        let source_pid = source.id();
        let stopping = Arc::new(AtomicBool::new(false));
        let feeding = Feed {
            source,
            sink: InputSink {
                inputs: inputs.clone(),
            },
            error_file: picker_error_file(cache),
            said_file,
            stopping: Arc::clone(&stopping),
        };
        let feeder = thread::spawn(move || {
            let _ = feed(feeding);
            let _ = inputs.send(Input::RowsEnded);
        });
        Ok(Self {
            source_pid,
            stopping,
            feeder: Some(feeder),
        })
    }
}

impl Drop for RowStream {
    fn drop(&mut self) {
        self.stopping.store(true, Ordering::SeqCst);
        if let Some(feeder) = self.feeder.take() {
            stop_the_group_again_until_feeding_ends_to_catch_late_forks(self.source_pid, &feeder);
            let _ = feeder.join();
        }
    }
}

struct InputSink {
    inputs: Sender<Input>,
}

impl RowSink for InputSink {
    fn take(&mut self, _: usize, row: PickerRow, _: &str) -> io::Result<()> {
        self.inputs
            .send(Input::RowsArrived(vec![row]))
            .map_err(|_| io::ErrorKind::BrokenPipe.into())
    }

    fn header(&mut self, _: &str) {}

    fn finish(self) -> Vec<PickerRow> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests;
