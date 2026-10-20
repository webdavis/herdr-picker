use std::net::Ipv4Addr;

// One of this Mac's IPv4 networks, as ifconfig lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Network {
    pub(super) interface: String,
    pub(super) address: Ipv4Addr,
    pub(super) prefix: u32,
}

impl Network {
    pub(super) fn range(&self) -> String {
        let mask = u32::MAX.checked_shl(32 - self.prefix).unwrap_or(0);
        format!(
            "{}/{}",
            Ipv4Addr::from(u32::from(self.address) & mask),
            self.prefix
        )
    }

    pub(super) fn broadcast(&self) -> Ipv4Addr {
        Ipv4Addr::from(u32::from(self.address) | u32::MAX.checked_shr(self.prefix).unwrap_or(0))
    }

    pub(super) fn holds(&self, address: Ipv4Addr) -> bool {
        let mask = u32::MAX.checked_shl(32 - self.prefix).unwrap_or(0);
        u32::from(address) & mask == u32::from(self.address) & mask
    }
}

// Every address ifconfig gives this Mac, so it is told apart from its neighbours.
pub(super) fn own_addresses(ifconfig: &str) -> Vec<String> {
    ifconfig
        .lines()
        .filter_map(
            |line| match line.split_whitespace().collect::<Vec<_>>()[..] {
                ["inet" | "inet6", address, ..] => {
                    Some(address.split('%').next().unwrap_or(address).to_string())
                }
                _ => None,
            },
        )
        .collect()
}

// The IPv4 networks worth scanning: none smaller than a /30 or bigger than a /16.
pub(super) fn networks(ifconfig: &str) -> Vec<Network> {
    let mut interface = "";
    let mut found = Vec::new();
    for line in ifconfig.lines() {
        if !line.starts_with(char::is_whitespace) {
            interface = line.split(':').next().unwrap_or_default();
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        let mask = parts
            .iter()
            .position(|part| *part == "netmask")
            .and_then(|at| parts.get(at + 1));
        let mask =
            mask.and_then(|mask| u32::from_str_radix(mask.trim_start_matches("0x"), 16).ok());
        if let (["inet", address, ..], Some(mask)) = (parts.as_slice(), mask) {
            let prefix = mask.count_ones();
            if let (Ok(address), true) = (
                address.parse(),
                interface != "lo0" && (16..31).contains(&prefix),
            ) {
                found.push(Network {
                    interface: interface.to_string(),
                    address,
                    prefix,
                });
            }
        }
    }
    found
}

// `netstat -rn -f inet`: the default route's gateway.
pub(super) fn gateway(routes: &str) -> String {
    routes
        .lines()
        .find_map(
            |line| match line.split_whitespace().collect::<Vec<_>>()[..] {
                ["default", gateway, ..] => Some(gateway.to_string()),
                _ => None,
            },
        )
        .unwrap_or_default()
}

#[cfg(test)]
mod tests;
