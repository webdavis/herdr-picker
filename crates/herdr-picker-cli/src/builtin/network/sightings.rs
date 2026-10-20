use serde_json::{Map, Value};

// Who saw a device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Source {
    UniFi,
    ThisMac,
    Bonjour,
    Scan,
}

impl Source {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::UniFi => "UniFi",
            Self::ThisMac => "This Mac",
            Self::Bonjour => "Bonjour",
            Self::Scan => "Scan",
        }
    }
}

// What one source saw of one device; the devices source merges them by MAC, then by address.
#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct Sighting {
    pub(super) ip: String,
    pub(super) mac: String,
    pub(super) name: String,
    pub(super) host: String,
    pub(super) neighbour: String,
    pub(super) vendor: String,
    pub(super) offers: Vec<String>,
    pub(super) unifi: Map<String, Value>,
}

// `arp -an`: `? (192.168.1.1) at 74:ac:b9:1:2:3 on en0 ifscope [ethernet]`.
pub(super) fn arp(table: &str) -> Vec<Sighting> {
    table
        .lines()
        .filter_map(|line| {
            let (name, rest) = line.split_once(" (")?;
            let (ip, rest) = rest.split_once(") at ")?;
            let (mac, _) = rest.split_once(" on ")?;
            (!mac.contains("incomplete")).then(|| Sighting {
                ip: ip.to_string(),
                mac: mac.to_string(),
                name: if name == "?" {
                    String::new()
                } else {
                    name.to_string()
                },
                neighbour: "answered recently".to_string(),
                ..Sighting::default()
            })
        })
        .collect()
}

// `ndp -an`: neighbour, MAC, interface, then its state; this Mac's own addresses are permanent.
pub(super) fn ndp(table: &str) -> Vec<Sighting> {
    table
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let [address, mac, interface, state @ ..] = parts.as_slice() else {
                return None;
            };
            let state = state.join(" ");
            let own =
                state.starts_with("permanent") || *interface == "lo0" || *mac == "(incomplete)";
            (address.contains(':') && parts.len() >= 5 && !own).then(|| Sighting {
                ip: address.split('%').next().unwrap_or(address).to_string(),
                mac: mac.to_string(),
                neighbour: state,
                ..Sighting::default()
            })
        })
        .collect()
}

// `dns-sd -B`: each announced instance's name and domain.
pub(super) fn browsed(output: &str) -> Vec<(String, String)> {
    output
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            (parts.len() >= 7 && parts[1] == "Add")
                .then(|| (parts[6..].join(" "), parts[4].to_string()))
        })
        .collect()
}

// `dns-sd -L`: the host an instance can be reached at.
pub(super) fn reached(output: &str) -> Option<String> {
    let (_, after) = output.split_once("can be reached at ")?;
    let (host, _) = after.split_once(':')?;
    Some(host.trim_end_matches('.').to_string())
}

#[cfg(test)]
mod tests;
