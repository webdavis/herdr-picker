use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

use super::ConfigError;
use super::pickers::PickerEntry;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct UnifiSettings {
    pub url: String,
    pub keychain_service: String,
    pub ca_file: Option<String>,
    pub server_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct BlueBubblesSettings {
    pub url: String,
    pub keychain_service: String,
}

type Tables = (Option<UnifiSettings>, Option<BlueBubblesSettings>);

pub(super) fn service_tables(
    entries: &mut BTreeMap<String, PickerEntry>,
    path: &Path,
) -> Result<Tables, ConfigError> {
    let unifi = owned_table(entries, path, "network", "unifi", |entry| {
        entry.unifi.take()
    })?;
    let bluebubbles = owned_table(entries, path, "apple", "bluebubbles", |entry| {
        entry.bluebubbles.take()
    })?;
    Ok((unifi, bluebubbles))
}

fn owned_table<T>(
    entries: &mut BTreeMap<String, PickerEntry>,
    path: &Path,
    owner: &'static str,
    table: &str,
    take: impl Fn(&mut PickerEntry) -> Option<T>,
) -> Result<Option<T>, ConfigError> {
    let mut found = None;
    for (name, entry) in entries.iter_mut() {
        match (take(entry), name == owner) {
            (Some(settings), true) => found = Some(settings),
            (Some(_), false) => {
                return Err(ConfigError::Misplaced {
                    path: path.to_owned(),
                    table: format!("[pickers.{name}.{table}]"),
                    owner,
                });
            }
            (None, _) => {}
        }
    }
    Ok(found)
}
