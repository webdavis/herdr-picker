use std::collections::BTreeMap;

use serde::Deserialize;

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

// The network picker's UniFi table, or the name of another picker that holds one.
pub(super) fn unifi_settings(
    entries: &mut BTreeMap<String, PickerEntry>,
) -> Result<Option<UnifiSettings>, String> {
    let mut found = None;
    for (name, entry) in entries.iter_mut() {
        match (entry.unifi.take(), name.as_str()) {
            (Some(unifi), "network") => found = Some(unifi),
            (Some(_), _) => return Err(name.clone()),
            (None, _) => {}
        }
    }
    Ok(found)
}
