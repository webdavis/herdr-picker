use std::path::Path;

use herdr_picker_adapters::read_captures;
use herdr_picker_domain::PickerRow;

use super::capture::{capture_row, saved_dir};
use super::diagnostics::diagnostic_rows;
use super::{JUSTFILES, nearest, notice, words};
use crate::builtin::Places;
use crate::builtin::recipes::recipes;
use crate::command::CommandError;

const CHECK_WORDS: [&str; 3] = ["lint", "check", "build"];

// The saved checks' messages first, then the checks that can run again to save new ones.
pub(super) fn errors(places: &Places) -> Result<Vec<PickerRow>, CommandError> {
    let mut rows = Vec::new();
    for saved in read_captures(&saved_dir(places), "diagnostics") {
        let argv: Vec<&str> = saved.argv.iter().map(String::as_str).collect();
        let exited = format!("{}: exit {}", saved.framework, saved.exit_code);
        rows.push(notice(&exited, &words(&argv)));
        let printed = format!("{}\n{}", saved.stdout, saved.stderr);
        rows.extend(diagnostic_rows(&printed, Path::new(&saved.cwd)));
    }
    let cwd = &places.cwd;
    if let Some(swift) = nearest(cwd, &["Package.swift"]) {
        let folder = swift.parent().unwrap_or(cwd);
        let build = words(&[
            "swift",
            "build",
            "--package-path",
            &folder.to_string_lossy(),
        ]);
        let label = "Run Swift build and save diagnostics";
        rows.push(capture_row(
            label,
            &build,
            folder,
            "diagnostics",
            "swift",
            places,
        ));
    }
    if let Some(rust) = nearest(cwd, &["Cargo.toml"]) {
        let manifest = rust.to_string_lossy();
        let check = words(&["cargo", "check", "--manifest-path", &manifest]);
        let check = format!("{check} --message-format=json");
        let label = "Run Cargo check and save diagnostics";
        rows.push(capture_row(
            label,
            &check,
            rust.parent().unwrap_or(cwd),
            "diagnostics",
            "rust",
            places,
        ));
    }
    if nearest(cwd, &JUSTFILES).is_some() {
        for recipe in recipes(cwd)? {
            let name = recipe.name.to_lowercase();
            if CHECK_WORDS.iter().any(|word| name.contains(word)) {
                let framework = format!("just-{}", recipe.name.replace("::", "-"));
                rows.push(capture_row(
                    &recipe.name,
                    &recipe.command,
                    cwd,
                    "diagnostics",
                    &framework,
                    places,
                ));
            }
        }
    }
    if rows.is_empty() {
        let detail = "Prepare a check here, or save a quickfix list with Ctrl-Q in any picker.";
        rows.push(notice("No saved results", detail));
    }
    Ok(rows)
}

#[cfg(test)]
mod tests;
