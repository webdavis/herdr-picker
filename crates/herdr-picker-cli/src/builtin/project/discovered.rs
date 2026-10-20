use std::path::Path;

use herdr_picker_adapters::read_captures;
use herdr_picker_domain::PickerRow;

use super::capture::{capture_row, saved_dir};
use super::frameworks::Framework;
use super::{command_row, nearest, notice};
use crate::builtin::Places;

// The tests each saved listing found, each run alone; a listing that failed says so first.
pub(super) fn discovered(places: &Places) -> Vec<PickerRow> {
    let mut rows = Vec::new();
    for saved in read_captures(&saved_dir(places), "tests") {
        if saved.exit_code != 0 {
            let failed = format!(
                "Discovery failed: {}, exit {}",
                saved.framework, saved.exit_code
            );
            rows.push(notice(
                &failed,
                &format!("{}\n{}", saved.stdout, saved.stderr),
            ));
        }
        let Some(framework) = Framework::named(&saved.framework) else {
            continue;
        };
        let folder = Path::new(&saved.cwd);
        let detail = format!("Framework: {}", saved.framework);
        rows.extend(framework.tests(&saved.stdout).into_iter().map(|test| {
            let command = framework.one(folder, &test);
            command_row(test, command, &detail)
        }));
    }
    rows
}

// For each kind of project the picker runs in: the row that saves its test listing, then the
// row that runs all its tests.
pub(super) fn discoveries(places: &Places) -> Vec<PickerRow> {
    let cwd = &places.cwd;
    let mut rows = Vec::new();
    for framework in Framework::ALL {
        let Some(marker) = nearest(cwd, framework.markers()) else {
            continue;
        };
        let folder = marker.parent().unwrap_or(cwd);
        let listing = framework.listing(folder);
        let label = framework.discover_label();
        rows.push(capture_row(
            label,
            &listing,
            folder,
            "tests",
            framework.name(),
            places,
        ));
        if let Some((label, command)) = framework.everything(folder) {
            rows.push(command_row(label.to_string(), command, ""));
        }
    }
    rows
}

#[cfg(test)]
mod tests;
