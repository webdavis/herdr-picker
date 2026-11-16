use std::path::Path;

use pwe_application::{StepFinder, StepOutcome, StepSetup};
use pwe_domain::Theme;
use pwe_protocol::PICKER_QUERY_VARIABLE;

use super::{Chrome, Helper, Placement, fzf_arguments};
use crate::live_step::save_step_command;
use crate::picker_files::{read_view, remove_run_files, write_view};
use crate::picker_run::{PickerRunError, run_picker};

pub struct FzfStepFinder<'a> {
    pub helper: Helper<'a>,
    pub cache: &'a Path,
    pub theme: Theme,
    pub placement: Placement,
}

impl StepFinder for FzfStepFinder<'_> {
    type Error = PickerRunError;

    fn find(&self, setup: StepSetup) -> Result<Option<StepOutcome>, PickerRunError> {
        let StepSetup {
            mut rows,
            query,
            header,
            view,
            live,
        } = setup;
        remove_run_files(self.cache);
        write_view(self.cache, &view).map_err(|failure| self.cannot_cache(failure))?;
        if live {
            save_step_command(self.cache, &rows).map_err(|failure| self.cannot_cache(failure))?;
            rows.env(PICKER_QUERY_VARIABLE, &query);
        }
        let chrome = Chrome {
            helper: self.helper,
            header: &header,
            description: &view.description,
            query: &query,
            live,
        };
        let arguments = fzf_arguments(self.theme, &chrome, self.placement);
        let found = run_picker(&mut rows, Path::new("fzf"), &arguments, self.cache);
        let view_left = read_view(self.cache);
        remove_run_files(self.cache);
        Ok(found?.map(|found| StepOutcome { found, view_left }))
    }
}

impl FzfStepFinder<'_> {
    fn cannot_cache(&self, failure: std::io::Error) -> PickerRunError {
        PickerRunError::CannotCache {
            path: self.cache.to_owned(),
            reason: failure.to_string(),
        }
    }
}
