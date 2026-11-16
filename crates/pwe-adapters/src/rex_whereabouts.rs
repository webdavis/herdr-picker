use std::ffi::OsString;

use pwe_application::{Whereabouts, WhereaboutsError};
use pwe_domain::OwnerPlace;
use pwe_protocol::parse_rex_whereabouts;

use crate::program_run::{ProgramRunError, run_on_path};

const OSASCRIPT: &str = "osascript";
const SCRIPT: &str = include_str!("rex_whereabouts/whereabouts.js");
const PERMISSION_REFUSED: &str = "(-1743)";

pub struct RexWhereabouts {
    app_name: String,
    path_variable: Option<OsString>,
}

impl RexWhereabouts {
    pub fn new(app_name: &str, path_variable: Option<OsString>) -> Self {
        Self {
            app_name: app_name.to_string(),
            path_variable,
        }
    }

    fn refusal(&self, stderr: &str) -> WhereaboutsError {
        if stderr.contains(PERMISSION_REFUSED) {
            return WhereaboutsError::PermissionRefused {
                app_name: self.app_name.clone(),
            };
        }
        WhereaboutsError::Unreadable(stderr.trim().to_string())
    }
}

impl Whereabouts for RexWhereabouts {
    fn owner_place(&self) -> Result<OwnerPlace, WhereaboutsError> {
        let output = run_on_path(
            OSASCRIPT,
            self.path_variable.as_deref(),
            &["-l", "JavaScript", "-", &self.app_name],
            SCRIPT,
        )
        .map_err(|failure| match failure {
            ProgramRunError::Missing => WhereaboutsError::ProgramMissing(OSASCRIPT.to_string()),
            ProgramRunError::Unrunnable(error) => WhereaboutsError::Unreadable(error.to_string()),
        })?;
        if !output.status.success() {
            return Err(self.refusal(&String::from_utf8_lossy(&output.stderr)));
        }
        let printed = String::from_utf8_lossy(&output.stdout);
        parse_rex_whereabouts(&printed).map_err(|_| {
            WhereaboutsError::Unreadable(format!("osascript printed: {}", printed.trim()))
        })
    }
}

#[cfg(test)]
mod tests;
