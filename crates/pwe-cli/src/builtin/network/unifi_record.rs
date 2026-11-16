use serde_json::Value;

use super::device_page::Clock;
use super::devices::Device;
use crate::builtin::clock::{parse_iso, stamp};
use crate::builtin::page::fields;

// UniFi fields the page already shows, or that name another record by its id.
const UNIFI_SHOWN: [&str; 18] = [
    "name",
    "ipAddress",
    "macAddress",
    "uplinkDeviceId",
    "type",
    "connectedAt",
    "id",
    "access",
    "site",
    "sites",
    "category",
    "features",
    "model",
    "state",
    "firmwareVersion",
    "firmwareUpdatable",
    "supported",
    "interfaces",
];

// UniFi's bookkeeping, each field named for what it is.
pub(super) fn unifi_fields(device: &Device) -> String {
    let unifi = |field: &str| device.unifi.get(field);
    let guest = match unifi("access") {
        Some(access) if access["type"] == "GUEST" => match access["authorized"] == true {
            true => "guest network, authorized".to_string(),
            false => "guest network, not yet authorized".to_string(),
        },
        _ => String::new(),
    };
    let sites = unifi("sites").and_then(Value::as_u64).unwrap_or(1);
    let mut rows = vec![
        ("UniFi record ID", unifi("id").map(text).unwrap_or_default()),
        ("UniFi access", guest),
        (
            "UniFi site",
            if sites > 1 {
                unifi("site").map(text).unwrap_or_default()
            } else {
                String::new()
            },
        ),
    ];
    for (field, value) in &device.unifi {
        let empty = matches!(value, Value::Null)
            || value == ""
            || value.as_array().is_some_and(Vec::is_empty)
            || value.as_object().is_some_and(|map| map.is_empty());
        if !UNIFI_SHOWN.contains(&field.as_str()) && !empty {
            rows.push((field.as_str(), text(value)));
        }
    }
    fields(&rows)
}

pub(super) fn text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

// UniFi's DEFAULT type is a placeholder for all other clients, so it says nothing worth a line.
const CONNECTIONS: [(&str, &str); 4] = [
    ("WIRED", "Wired"),
    ("WIRELESS", "Wi-Fi"),
    ("VPN", "VPN"),
    ("TELEPORT", "Teleport VPN"),
];
pub(super) fn since(stamped: &str, clock: Clock) -> String {
    let Some(then) = parse_iso(stamped) else {
        return stamped.to_string();
    };
    let hours = (clock.now - then).max(0) / 3600;
    let ago = match hours {
        0 => "under an hour".to_string(),
        48.. => format!("{} days", hours / 24),
        _ => format!("{hours} hours"),
    };
    format!("{} ({ago})", stamp(then, clock.offset))
}

pub(super) fn connection(kind: &str) -> Option<&'static str> {
    CONNECTIONS
        .iter()
        .find(|(code, _)| *code == kind)
        .map(|(_, shown)| *shown)
}
