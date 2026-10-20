use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use herdr_picker_domain::PickerRow;
use serde_json::Value;

use super::capture::MANIFESTS;
use super::{command_row, nearest, notice, words};
use crate::builtin::Places;
use crate::builtin::paint::dim;
use crate::builtin::rows::{folder_row, shown};

// The nearest package.json's scripts with "test" in their name, each run by npm in its folder.
pub(super) fn npm_scripts(cwd: &Path) -> Vec<PickerRow> {
    let Some(package) = nearest(cwd, &["package.json"]) else {
        return Vec::new();
    };
    let read = fs::read_to_string(&package).ok();
    let Some(parsed) = read.and_then(|text| serde_json::from_str::<Value>(&text).ok()) else {
        return vec![notice("Invalid package.json", &package.to_string_lossy())];
    };
    let folder = package.parent().unwrap_or(cwd).to_string_lossy();
    let scripts = parsed.get("scripts").and_then(Value::as_object);
    scripts
        .into_iter()
        .flatten()
        .filter(|(name, _)| name.to_lowercase().contains("test"))
        .filter_map(|(name, script)| {
            let command = words(&["npm", "--prefix", &folder, "run", name]);
            Some(command_row(
                format!("npm: {name}"),
                command,
                script.as_str()?,
            ))
        })
        .collect()
}

// Each project in a folder below the picker's, outermost only, which Enter moves to.
pub(super) fn nested_projects(files: &[PathBuf], places: &Places) -> Vec<PickerRow> {
    let mut projects: BTreeMap<&Path, Vec<&str>> = BTreeMap::new();
    for file in files {
        let (Some(folder), Some(name)) = (file.parent(), file.file_name()) else {
            continue;
        };
        let name = name.to_str().unwrap_or_default();
        if folder != places.cwd && folder.starts_with(&places.cwd) && MANIFESTS.contains(&name) {
            projects.entry(folder).or_default().push(name);
        }
    }
    let mut outermost: Vec<&Path> = Vec::new();
    let mut rows = Vec::new();
    for (folder, mut markers) in projects {
        if outermost.iter().any(|outer| folder.starts_with(outer)) {
            continue;
        }
        outermost.push(folder);
        markers.sort_unstable();
        let mut row = folder_row(folder, places);
        let shown = shown(folder, &places.cwd);
        row.label = format!("Project: {shown}  {}", dim(&markers.join(", ")));
        rows.push(row);
    }
    rows
}

#[cfg(test)]
mod tests;
