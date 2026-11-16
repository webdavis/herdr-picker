use serde_json::{Map, Value};

use super::naming::{display_name, is_loopback, primary_ip};
use super::sightings::{Sighting, Source};

// Everything the sources saw of one device.
#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct Device {
    pub(super) ips: Vec<String>,
    pub(super) names: Vec<(Source, String)>,
    pub(super) sources: Vec<Source>,
    pub(super) mac: String,
    pub(super) offers: Vec<String>,
    pub(super) unifi: Map<String, Value>,
    pub(super) host: String,
    pub(super) neighbour: String,
    pub(super) vendor: String,
    // The UniFi gear it connects through: its name and address.
    pub(super) through: Option<(String, String)>,
    // This Mac's computer name, when the device is the Mac in use.
    pub(super) this_mac: Option<String>,
}

#[derive(Default)]
pub(super) struct Devices(pub(super) Vec<Device>);

impl Devices {
    pub(super) fn add(&mut self, source: Source, seen: Sighting) {
        let mac = if seen.mac.contains(':') {
            seen.mac.to_lowercase()
        } else {
            String::new()
        };
        let found = self
            .0
            .iter()
            .position(|device| !mac.is_empty() && device.mac == mac);
        let found = found.or_else(|| {
            self.0
                .iter()
                .position(|device| !seen.ip.is_empty() && device.ips.contains(&seen.ip))
        });
        let at = found.unwrap_or_else(|| {
            self.0.push(Device::default());
            self.0.len() - 1
        });
        let device = &mut self.0[at];
        let other = Device {
            ips: [seen.ip].into_iter().filter(|ip| !ip.is_empty()).collect(),
            names: [(source, seen.name)]
                .into_iter()
                .filter(|(_, name)| !name.is_empty())
                .collect(),
            sources: vec![source],
            mac,
            offers: seen.offers,
            unifi: seen.unifi,
            host: seen.host,
            neighbour: seen.neighbour,
            vendor: seen.vendor,
            ..Device::default()
        };
        absorb(device, other, true);
    }

    // Every address of this Mac is one device, whichever source reported it.
    pub(super) fn mark_this_mac(&mut self, own: &[String], computer_name: &str) {
        let is_mine = |device: &Device| {
            device
                .ips
                .iter()
                .any(|ip| own.contains(ip) || is_loopback(ip))
        };
        let Some(first) = self.0.iter().position(is_mine) else {
            return;
        };
        let (mine, others): (Vec<Device>, Vec<Device>) =
            std::mem::take(&mut self.0).into_iter().partition(is_mine);
        self.0 = others;
        let mut mine = mine.into_iter();
        if let Some(mut me) = mine.next() {
            mine.for_each(|other| absorb(&mut me, other, false));
            me.this_mac = Some(computer_name.to_string());
            self.0.insert(first, me);
        }
    }

    // Each client learns the UniFi gear it is connected through.
    pub(super) fn link_uplinks(&mut self) {
        let gear: Vec<(String, (String, String))> = self
            .0
            .iter()
            .filter(|device| {
                device.unifi.get("category").and_then(Value::as_str) == Some("devices")
            })
            .filter_map(|device| {
                Some((
                    device.unifi.get("id")?.as_str()?.to_string(),
                    (display_name(device), primary_ip(device)),
                ))
            })
            .collect();
        for device in &mut self.0 {
            let uplink = device.unifi.get("uplinkDeviceId").and_then(Value::as_str);
            device.through = gear
                .iter()
                .find(|(id, _)| Some(id.as_str()) == uplink)
                .map(|(_, gear)| gear.clone());
        }
    }
}

// Adds what `other` knows to `device`. A fresh sighting's host, neighbour state and vendor
// replace older ones; merging this Mac's addresses keeps the device's own.
fn absorb(device: &mut Device, other: Device, newer_wins: bool) {
    unique(&mut device.ips, other.ips);
    unique(&mut device.offers, other.offers);
    for (source, name) in other.names {
        if !device.names.iter().any(|(known, _)| *known == source) {
            device.names.push((source, name));
        }
    }
    unique(&mut device.sources, other.sources);
    device.unifi.extend(other.unifi);
    if device.mac.is_empty() {
        device.mac = other.mac;
    }
    for (mine, theirs) in [
        (&mut device.host, other.host),
        (&mut device.neighbour, other.neighbour),
        (&mut device.vendor, other.vendor),
    ] {
        if !theirs.is_empty() && (mine.is_empty() || newer_wins) {
            *mine = theirs;
        }
    }
    device.through = device.through.take().or(other.through);
}

fn unique<T: PartialEq>(into: &mut Vec<T>, from: Vec<T>) {
    for item in from {
        if !into.contains(&item) {
            into.push(item);
        }
    }
}

#[cfg(test)]
mod tests;
