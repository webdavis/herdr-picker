use std::net::IpAddr;

use serde_json::Value;

use super::devices::Device;
use super::sightings::Source;

// The order a device's name is taken in: the controller knows best, a scan least.
const SOURCES: [Source; 4] = [
    Source::UniFi,
    Source::ThisMac,
    Source::Bonjour,
    Source::Scan,
];
const UNIFI_ROLES: [(&str, &str); 3] = [
    ("gateway", "router"),
    ("accessPoint", "access point"),
    ("switching", "switch"),
];
// UniFi's features list omits routing, so the model prefix says which gear is the router.
const ROUTER_MODELS: [&str; 5] = ["UDR", "UDM", "UXG", "UCG", "USG"];

pub(super) fn is_loopback(ip: &str) -> bool {
    ip.parse::<IpAddr>().is_ok_and(|ip| ip.is_loopback())
}

// The address to paste and ssh to: a routable IPv4 one when there is a choice.
pub(super) fn primary_ip(device: &Device) -> String {
    let mut ranked: Vec<&String> = device.ips.iter().collect();
    ranked.sort_by_key(|ip| (is_loopback(ip), ip.contains(':'), ip.as_str()));
    ranked.first().map_or(String::new(), |ip| ip.to_string())
}

pub(super) fn display_name(device: &Device) -> String {
    if let Some(name) = device.this_mac.as_ref().filter(|name| !name.is_empty()) {
        return name.clone();
    }
    let named = SOURCES
        .iter()
        .find_map(|source| device.names.iter().find(|(seen, _)| seen == source));
    match named {
        Some((_, name)) => name.clone(),
        None if !device.host.is_empty() => device.host.clone(),
        None => [primary_ip(device), device.mac.clone()]
            .into_iter()
            .find(|text| !text.is_empty())
            .unwrap_or_default(),
    }
}

pub(super) fn role(device: &Device) -> String {
    if device.this_mac.is_some() {
        return "this Mac".to_string();
    }
    if device.unifi.get("category").and_then(Value::as_str) != Some("devices") {
        return String::new();
    }
    let mut features: Vec<&str> = device
        .unifi
        .get("features")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    let model = device
        .unifi
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_uppercase();
    if ROUTER_MODELS.iter().any(|router| model.starts_with(router)) {
        features.push("gateway");
    }
    let roles: Vec<&str> = UNIFI_ROLES
        .iter()
        .filter(|(feature, _)| features.contains(feature))
        .map(|(_, role)| *role)
        .collect();
    match roles.is_empty() {
        true => "UniFi gear".to_string(),
        false => roles.join(", "),
    }
}
