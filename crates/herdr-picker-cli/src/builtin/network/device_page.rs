use serde_json::Value;

use super::bonjour::plain;
use super::devices::Device;
use super::naming::{display_name, role};
use super::unifi_record::{connection, since, text, unifi_fields};
use crate::builtin::page::fields;

// The clock a page reads ages and local times from.
#[derive(Debug, Clone, Copy)]
pub(super) struct Clock {
    pub(super) now: i64,
    pub(super) offset: i64,
}

pub(super) fn page(device: &Device, gateway: &str, clock: Clock) -> String {
    let unifi = |field: &str| device.unifi.get(field).map(text).unwrap_or_default();
    let title = display_name(device);
    let others: Vec<String> = device
        .names
        .iter()
        .filter(|(_, name)| *name != title)
        .map(|(source, name)| format!("{name} ({})", source.label()))
        .collect();
    let through = device
        .through
        .as_ref()
        .map(|(name, ip)| joined(name, ip))
        .unwrap_or_default();
    let has_v4 = device.ips.iter().any(|ip| !ip.contains(':'));
    let details = fields(&[
        ("Also known as", others.join(", ")),
        ("IP address", device.ips.join(", ")),
        ("MAC address", device.mac.clone()),
        ("Hostname", device.host.clone()),
        ("Vendor", device.vendor.clone()),
        (
            "Connection",
            connection(&unifi("type")).unwrap_or_default().to_string(),
        ),
        ("Connected through", through),
        ("Connected since", since(&unifi("connectedAt"), clock)),
        (
            "Gateway",
            if has_v4 {
                gateway.to_string()
            } else {
                String::new()
            },
        ),
        (
            "Found by",
            device
                .sources
                .iter()
                .map(|source| source.label())
                .collect::<Vec<_>>()
                .join(", "),
        ),
        ("Last answered", device.neighbour.clone()),
    ]);
    let offers: Vec<(&str, String)> = device
        .offers
        .iter()
        .map(|code| (plain(code), code.clone()))
        .collect();
    let mut sections = vec![
        format!("{title}\n{}", sentence(device, clock).trim_end_matches('.')),
        format!("Details\n{details}"),
    ];
    if !offers.is_empty() {
        sections.push(format!("Services announced\n{}", fields(&offers)));
    }
    if !device.unifi.is_empty() {
        sections.push(format!("From UniFi\n{}", unifi_fields(device)));
    }
    sections.join("\n\n")
}

pub(super) fn sentence(device: &Device, clock: Clock) -> String {
    let unifi = |field: &str| device.unifi.get(field).map(text).unwrap_or_default();
    if device.this_mac.is_some() {
        return "This is the Mac you are using.".to_string();
    }
    if unifi("category") == "devices" {
        let mut parts = vec![format!("UniFi {}", role(device))];
        if !unifi("model").is_empty() {
            parts[0].push_str(&format!(" ({})", unifi("model")));
        }
        parts.extend(Some(unifi("state").to_lowercase()).filter(|state| !state.is_empty()));
        if !unifi("firmwareVersion").is_empty() {
            let updatable = device.unifi.get("firmwareUpdatable") == Some(&Value::Bool(true));
            let note = if updatable {
                ", update available"
            } else {
                ", up to date"
            };
            parts.push(format!("firmware {}{note}", unifi("firmwareVersion")));
        }
        return parts.join(", ") + ".";
    }
    let mut bits: Vec<String> = Vec::new();
    bits.extend(
        Some(unifi("type"))
            .filter(|kind| !kind.is_empty())
            .map(|kind| connection(&kind).unwrap_or(&kind).to_string())
            .filter(|kind| !kind.is_empty()),
    );
    if let Some((name, ip)) = &device.through {
        bits.push(format!("connected through {}", joined(name, ip)));
    }
    if !unifi("connectedAt").is_empty() {
        bits.push(format!("since {}", since(&unifi("connectedAt"), clock)));
    }
    match bits.is_empty() {
        true => format!(
            "Seen by {}.",
            device
                .sources
                .iter()
                .map(|source| source.label())
                .collect::<Vec<_>>()
                .join(" and ")
        ),
        false => bits.join(", ") + ".",
    }
}

fn joined(name: &str, ip: &str) -> String {
    if ip.is_empty() {
        name.to_string()
    } else {
        format!("{name} ({ip})")
    }
}

#[cfg(test)]
mod tests;
