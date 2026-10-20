use std::path::Path;

use herdr_picker_adapters::read_captures;
use herdr_picker_domain::PickerRow;

use super::capture::saved_dir;
use super::diagnostics::diagnostic_rows;
use super::{notice, words};
use crate::builtin::Places;

// The saved test and check runs that failed, each followed by its messages, for handing to an agent.
pub(in crate::builtin) fn failures(places: &Places) -> Vec<PickerRow> {
    let mut rows = Vec::new();
    for category in ["tests", "diagnostics"] {
        for saved in read_captures(&saved_dir(places), category) {
            if saved.exit_code == 0 {
                continue;
            }
            let argv: Vec<&str> = saved.argv.iter().map(String::as_str).collect();
            rows.push(notice(
                &format!("{category} {}: exit {}", saved.framework, saved.exit_code),
                &words(&argv),
            ));
            let printed = format!("{}\n{}", saved.stdout, saved.stderr);
            rows.extend(diagnostic_rows(&printed, Path::new(&saved.cwd)));
        }
    }
    if rows.is_empty() {
        let detail = "Run a check from the tests picker's errors source to save its failures.";
        rows.push(notice("No saved failures", detail));
    }
    rows
}
