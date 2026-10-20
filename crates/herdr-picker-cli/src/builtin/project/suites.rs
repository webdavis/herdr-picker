use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use herdr_picker_adapters::{Entry, Walk, walk};
use herdr_picker_domain::{Action, Location, PickerRow};

use super::{JUSTFILES, command_row, nearest, notice, root, words};
use crate::builtin::Places;
use crate::builtin::git::{output, strings};
use crate::builtin::recipes::{recipe_row, recipes};
use crate::builtin::rows::key;
use crate::command::CommandError;

const SCRIPT_FOLDERS: [&str; 3] = ["unit", "integration", "e2e"];

pub(super) fn suites(places: &Places) -> Result<Vec<PickerRow>, CommandError> {
    let cwd = &places.cwd;
    let mut rows = Vec::new();
    if let Some(swift) = nearest(cwd, &["Package.swift"]) {
        let folder = swift.parent().unwrap_or(cwd).to_string_lossy();
        let command = words(&["swift", "test", "--package-path", &folder]);
        rows.push(command_row("Run all Swift tests".into(), command, ""));
    }
    if let Some(rust) = nearest(cwd, &["Cargo.toml"]) {
        let command = words(&["cargo", "test", "--manifest-path", &rust.to_string_lossy()]);
        rows.push(command_row("Run all Rust tests".into(), command, ""));
    }
    let root = root(cwd);
    for path in files(&root, places)
        .iter()
        .filter(|path| path.starts_with(cwd))
    {
        rows.extend(bash_rows(path, &root));
    }
    if nearest(cwd, &JUSTFILES).is_some() {
        let tests: Vec<_> = recipes(cwd)?
            .into_iter()
            .filter(|recipe| recipe.name.to_lowercase().contains("test"))
            .collect();
        let width = tests.iter().map(|recipe| recipe.name.len()).max();
        let width = width.unwrap_or_default();
        rows.extend(tests.iter().map(|recipe| recipe_row(recipe, width, places)));
    }
    if rows.is_empty() {
        let detail = "Cargo, Swift, Bashunit and test recipes in a justfile are supported.";
        rows.push(notice("No supported tests found", detail));
    }
    Ok(rows)
}

// The repository's files Git does not ignore, or every file under a folder outside one.
fn files(root: &Path, places: &Places) -> Vec<PathBuf> {
    let mut listing = Command::new("git");
    listing.arg("-C").arg(root);
    listing.args(["ls-files", "-co", "--exclude-standard", "-z"]);
    let mut found: Vec<PathBuf> = match output(&mut listing) {
        Ok(listed) => strings(&listed)
            .iter()
            .map(|name| root.join(name))
            .collect(),
        Err(_) => {
            let mut found = Vec::new();
            let every = Walk {
                root: root.to_path_buf(),
                entry: Entry::Files,
                hidden: false,
                depth: None,
            };
            let _ = walk(&every, places.fd.as_deref(), |path| {
                found.push(path.to_path_buf());
                Ok(())
            });
            found
        }
    };
    found.sort();
    found.dedup();
    found
}

// A script in a unit, integration or e2e folder runs whole; a .test.sh file's tests run one by one.
fn bash_rows(path: &Path, root: &Path) -> Vec<PickerRow> {
    let shown = path.strip_prefix(root).unwrap_or(path).to_string_lossy();
    let full = path.to_string_lossy();
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    if name.ends_with(".test.sh") {
        let text = fs::read_to_string(path).unwrap_or_default();
        return text
            .lines()
            .enumerate()
            .filter_map(|(index, line)| Some((bash_test(line)?, index + 1)))
            .map(|(test, number)| {
                let command = words(&["bashunit", &format!("{full}:{number}")]);
                let mut row = command_row(format!("{shown}: {test}"), command, "");
                let at = Location {
                    path: full.to_string(),
                    line: u32::try_from(number).ok(),
                };
                row.keys.push((key("alt-e"), Action::Edit(vec![at])));
                row
            })
            .collect();
    }
    let in_script_folder = path
        .parent()
        .and_then(Path::file_name)
        .is_some_and(|folder| SCRIPT_FOLDERS.iter().any(|wanted| folder == *wanted));
    match name.ends_with(".sh") && in_script_folder {
        true => vec![command_row(shown.to_string(), words(&["bash", &full]), "")],
        false => Vec::new(),
    }
}

// The name of a test function a line declares, as `test_name() {` or `function test_name () {`.
fn bash_test(line: &str) -> Option<&str> {
    let line = line.trim_start();
    let line = match line.strip_prefix("function") {
        Some(rest) if rest.starts_with(char::is_whitespace) => rest.trim_start(),
        _ => line,
    };
    let end = line
        .find(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .unwrap_or(line.len());
    let (name, rest) = line.split_at(end);
    let rest = rest.trim_start().strip_prefix('(')?.trim_start();
    let opens = rest.strip_prefix(')')?.trim_start().starts_with('{');
    (opens && name.len() > "test_".len() && name.starts_with("test_")).then_some(name)
}

#[cfg(test)]
mod tests;
