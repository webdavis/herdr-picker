use std::fmt::Display;
use std::process::Command;

use pwe_domain::finder::Found;
use pwe_domain::{PickerRow, ViewState};

pub struct StepSetup {
    pub rows: Command,
    pub query: String,
    pub header: String,
    pub view: ViewState,
    pub live: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepOutcome {
    pub found: Found<PickerRow>,
    pub view_left: ViewState,
}

pub trait StepFinder {
    type Error: Display;

    fn find(&self, setup: StepSetup) -> Result<Option<StepOutcome>, Self::Error>;
}
