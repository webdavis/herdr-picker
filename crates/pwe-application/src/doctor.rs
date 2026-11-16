use pwe_domain::{DoctorCheck, OwnerPlace};

use crate::{Whereabouts, WhereaboutsError};

const AUTOMATION_PERMISSION: &str = "automation-permission";

pub fn automation_permission_check(whereabouts: &dyn Whereabouts, app_name: &str) -> DoctorCheck {
    let failed = |finding: String, what_to_do: String| {
        DoctorCheck::failed(AUTOMATION_PERMISSION, finding, what_to_do)
    };
    match whereabouts.owner_place() {
        Ok(OwnerPlace::Closed) => failed(
            format!("{app_name} is not running, so pwe could not ask it."),
            format!("Open {app_name} and run pwe doctor again."),
        ),
        Ok(_) => DoctorCheck::passed(
            AUTOMATION_PERMISSION,
            format!("pwe may ask {app_name} which session you see."),
        ),
        Err(WhereaboutsError::PermissionRefused { .. }) => failed(
            format!("macOS refused pwe permission to ask {app_name} which session you see."),
            format!(
                "Open System Settings, then Privacy & Security, then Automation, and turn on {app_name} under the program that runs pwe."
            ),
        ),
        Err(refusal) => failed(
            refusal.to_string(),
            format!("Check that osascript runs and that {app_name} is the name of your Rex app."),
        ),
    }
}
