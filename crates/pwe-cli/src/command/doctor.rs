use std::env;

use pwe_adapters::{RexWhereabouts, read_picker_config};
use pwe_application::automation_permission_check;
use pwe_domain::CheckResult;
use pwe_protocol::{doctor_report_json, doctor_report_words};

use super::{CommandError, config_path, failed, prints_json};

pub(super) fn execute(words: &[&str]) -> Option<Result<(), CommandError>> {
    Some(match words {
        ["doctor", options @ ..] => prints_json(options).and_then(doctor),
        _ => return None,
    })
}

fn doctor(as_json: bool) -> Result<(), CommandError> {
    let app_name = read_picker_config(&config_path())
        .map_err(failed)?
        .rex
        .app_name;
    let whereabouts = RexWhereabouts::new(&app_name, env::var_os("PATH"));
    let checks = [automation_permission_check(&whereabouts, &app_name)];
    if as_json {
        println!("{}", doctor_report_json(&checks));
    } else {
        print!("{}", doctor_report_words(&checks));
    }
    let failures = checks
        .iter()
        .filter(|check| check.result == CheckResult::Failed)
        .count();
    match failures {
        0 => Ok(()),
        _ => Err(failed(format!(
            "{failures} of {} checks failed",
            checks.len()
        ))),
    }
}
