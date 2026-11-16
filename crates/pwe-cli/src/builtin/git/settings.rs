use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::process::Command;

use pwe_domain::PickerRow;

use super::{Repo, output, strings};
use crate::builtin::rows::RowWriter;
use crate::builtin::{Places, finder_closing_is_no_failure};
use crate::command::CommandError;

mod files;
mod line;
mod row;

use row::setting_row;

use files::chezmoi_sources;

const SECRET_WORDS: [&str; 5] = ["password", "token", "secret", "extraheader", "credential"];

pub(super) struct Entry {
    scope: String,
    pub(super) origin: String,
    name: String,
    value: String,
}

pub(super) fn settings(
    places: &Places,
    rows: &mut RowWriter<impl Write>,
) -> Result<(), CommandError> {
    let listed = output(Command::new("git").arg("-C").arg(&places.cwd).args([
        "config",
        "--null",
        "--list",
        "--show-origin",
        "--show-scope",
    ]))?;
    let entries: Vec<Entry> = strings(&listed)
        .as_chunks::<3>()
        .0
        .iter()
        .map(|found| {
            let (name, value) = found[2].split_once('\n').unwrap_or((&found[2], ""));
            Entry {
                scope: found[0].clone(),
                origin: found[1].clone(),
                name: name.to_string(),
                value: value.to_string(),
            }
        })
        .collect();
    let mut seen: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    let mut winner: BTreeMap<String, &str> = BTreeMap::new();
    for entry in &entries {
        seen.entry(entry.name.to_lowercase())
            .or_default()
            .push(&entry.origin);
        winner.insert(entry.name.to_lowercase(), &entry.scope);
    }
    let several: BTreeSet<&String> = seen
        .iter()
        .filter(|(_, origins)| origins.len() != origins.iter().collect::<BTreeSet<_>>().len())
        .map(|(name, _)| name)
        .collect();
    let base = Repo::at(&places.cwd).map_or_else(|_| places.cwd.clone(), |repo| repo.root);
    let managed = chezmoi_sources(&entries, &base);
    let listed = entries.iter().enumerate().map(|(index, entry)| {
        let lower = entry.name.to_lowercase();
        let later = entries[index + 1..]
            .iter()
            .any(|other| other.name.to_lowercase() == lower);
        let note = match !several.contains(&lower) && later {
            true => format!("overridden by {}", winner[&lower]),
            false => String::new(),
        };
        setting_row(entry, &note, several.contains(&lower), &managed, &base)
    });
    let listed: Vec<PickerRow> = listed.collect();
    finder_closing_is_no_failure(listed.iter().try_for_each(|row| rows.row(row)))
}

#[cfg(test)]
mod tests;
