use std::ffi::OsString;
use std::io;
use std::path::Path;
use std::sync::mpsc;

use pwe_application::{StepFinder, StepOutcome, StepSetup};
use pwe_domain::Theme;
use pwe_domain::finder::Finder;

use super::{
    Arrivals, FinderTerminal, FrizbeeMatcher, PreviewWorker, RowStream, StepRun, run_step,
};
use crate::picker_files::remove_run_files;
use crate::picker_run::PickerRunError;

pub struct TerminalFinder<'a, T> {
    pub cache: &'a Path,
    pub theme: Theme,
    pub quickfix: &'a Path,
    pub path_variable: Option<OsString>,
    pub open_terminal: fn() -> io::Result<T>,
}

impl<T: FinderTerminal> StepFinder for TerminalFinder<'_, T> {
    type Error = PickerRunError;

    fn find(&self, setup: StepSetup) -> Result<Option<StepOutcome>, PickerRunError> {
        let StepSetup {
            mut rows,
            query,
            view,
            ..
        } = setup;
        remove_run_files(self.cache);
        let (inputs, arrived) = mpsc::channel();
        let previews = PreviewWorker::start(inputs.clone(), self.path_variable.clone());
        let stream = RowStream::start(&mut rows, self.cache, inputs)?;
        let mut terminal = (self.open_terminal)()
            .map_err(|failure| PickerRunError::TerminalFailed(failure.to_string()))?;
        let run = StepRun {
            finder: Finder::new(FrizbeeMatcher)
                .with_view(view)
                .with_query(&query),
            arrivals: Arrivals::new(arrived),
            previews: &previews,
            theme: self.theme,
            quickfix: self.quickfix,
        };
        let outcome = run_step(&mut terminal, run);
        drop(terminal);
        drop(stream);
        remove_run_files(self.cache);
        outcome
    }
}
