use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

use super::ConfigError;
use super::pickers::PickerEntry;

// Where the network picker reaches a UniFi controller. The API key stays in the Keychain item
// named here; the picker reads it for each request and never writes it down.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct UnifiSettings {
    pub url: String,
    pub keychain_service: String,
    pub ca_file: Option<String>,
    // The name the controller's certificate is made out to, when it answers at another address.
    pub server_name: Option<String>,
}

// Where the apple picker reaches a BlueBubbles server. The server password stays in the Keychain
// item named here, the same way as UniFi's key.
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

// One picker's service table, taken out of every picker's entry so no other one keeps it.
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
