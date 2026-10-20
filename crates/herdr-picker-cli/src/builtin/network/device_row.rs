use std::env;
use std::net::Ipv4Addr;

use herdr_picker_domain::{Action, PickerCall, PickerRow, Preview};

use super::device_page::{Clock, page};
use super::devices::Device;
use super::interfaces::Network;
use super::naming::{display_name, primary_ip, role};
use crate::builtin::page::{own_program, words};
use crate::builtin::paint::{dim, mauve};
use crate::builtin::rows::key;

// Enter pastes the device's address; the preview pings it and looks up its name when shown.
pub(super) fn device_row(
    device: &Device,
    gateway: &str,
    networks: &[Network],
    clock: Clock,
) -> PickerRow {
    let name = display_name(device);
    let address = [primary_ip(device), device.mac.clone()]
        .into_iter()
        .find(|text| !text.is_empty())
        .unwrap_or_default();
    let tag = role(device);
    let mut label = name.clone();
    if !address.is_empty() && address != name {
        label.push_str(&format!("  {}", dim(&address)));
    }
    if !tag.is_empty() {
        label.push_str(&format!("  {}", mauve(&tag)));
    }
    let text = page(device, gateway, clock);
    let ip = primary_ip(device);
    let preview = match (
        device.this_mac.is_some() || ip.is_empty(),
        env::current_exe(),
    ) {
        (false, Ok(program)) => Preview::Command(vec![
            program.to_string_lossy().into_owned(),
            "network".into(),
            "ping".into(),
            ip.clone(),
            text,
        ]),
        _ => Preview::Text(text),
    };
    let scan = |state: Option<&str>| {
        Action::Next(PickerCall {
            name: "network-scan".into(),
            args: Vec::new(),
            state: state.map(str::to_string),
            query: None,
        })
    };
    let mut keys = Vec::new();
    if !address.is_empty() {
        keys.push((key("ctrl-s"), Action::Insert(words(&["ssh", &address]))));
    }
    keys.extend([
        (key("ctrl-x"), scan(None)),
        (key("alt-x"), scan(Some("prepare"))),
        (key("ctrl-g"), Action::Reload),
    ]);
    let host = [device.host.clone(), ip.clone()]
        .into_iter()
        .find(|host| !host.is_empty());
    if let Some(host) = host {
        let secure = device.offers.iter().any(|offer| offer == "_https._tcp");
        keys.push((
            key("ctrl-o"),
            Action::Open(format!(
                "{}://{host}",
                if secure { "https" } else { "http" }
            )),
        ));
    }
    let targets = wake_targets(&ip, networks);
    if !device.mac.is_empty() && !targets.is_empty() {
        let mut wake = vec!["network", "wake", &device.mac];
        wake.extend(targets.iter().map(String::as_str));
        let wake = format!("{} {}", own_program(), words(&wake));
        keys.push((key("ctrl-y"), Action::Run(wake)));
    }
    PickerRow {
        label,
        preview: Some(preview),
        enter: Action::Insert(address),
        keys,
        group: device
            .sources
            .first()
            .map(|source| source.label().to_string()),
    }
}

// macOS routes a broadcast only to one network's broadcast address, so the packet goes to the
// network the device is on, or to every one of this Mac's networks when that is unknown.
fn wake_targets(ip: &str, networks: &[Network]) -> Vec<String> {
    let broadcasts = |holding: &dyn Fn(&Network) -> bool| -> Vec<String> {
        networks
            .iter()
            .filter(|network| holding(network))
            .map(|network| network.broadcast().to_string())
            .collect()
    };
    let own = match ip.parse::<Ipv4Addr>() {
        Ok(address) => broadcasts(&|network| network.holds(address)),
        Err(_) => Vec::new(),
    };
    if own.is_empty() {
        broadcasts(&|_| true)
    } else {
        own
    }
}

#[cfg(test)]
mod tests;
