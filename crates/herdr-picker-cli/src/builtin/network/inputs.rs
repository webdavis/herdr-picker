use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

// What the user typed for one of a command's blanks, checked before it reaches a shell.
pub(super) fn validate(name: &str, value: &str) -> Result<String, String> {
    let value = value.trim();
    let invalid = || format!("Invalid {name}");
    match name {
        "port" | "count" | "duration" => match value.parse::<u32>() {
            Ok(number) if number >= 1 && (name != "port" || number <= 65_535) => {
                Ok(value.to_string())
            }
            _ if name == "port" => Err("port must be a positive integer up to 65535".to_string()),
            _ => Err(format!("{name} must be a positive integer")),
        },
        "ip" => value
            .parse::<IpAddr>()
            .map(|_| value.to_string())
            .map_err(|_| invalid()),
        "cidr" => network(value),
        "host" if value.contains(':') => {
            let (address, scope) = value.split_once('%').unwrap_or((value, ""));
            let scoped = scope
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "_.-".contains(c));
            match address.parse::<Ipv6Addr>() {
                Ok(_) if scoped => Ok(value.to_string()),
                _ => Err(invalid()),
            }
        }
        "host" | "interface" => {
            let mut characters = value.chars();
            let first = characters
                .next()
                .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_');
            let rest = characters.all(|c| c.is_ascii_alphanumeric() || "_.:%-".contains(c));
            match first && rest {
                true => Ok(value.to_string()),
                false => Err(invalid()),
            }
        }
        _ if value.is_empty() || value.chars().any(char::is_control) => Err(invalid()),
        _ => Ok(value.to_string()),
    }
}

// A range such as 192.168.1.77/24, given back as the network it is in: 192.168.1.0/24.
fn network(value: &str) -> Result<String, String> {
    let (address, prefix) = value
        .split_once('/')
        .ok_or("Choose an explicit subnet, such as 192.168.1.0/24")?;
    let invalid = || "Invalid cidr".to_string();
    let prefix: u32 = prefix.parse().map_err(|_| invalid())?;
    match address.parse::<IpAddr>().map_err(|_| invalid())? {
        IpAddr::V4(address) if prefix <= 32 => {
            let mask = u32::MAX.checked_shl(32 - prefix).unwrap_or(0);
            Ok(format!(
                "{}/{prefix}",
                Ipv4Addr::from(u32::from(address) & mask)
            ))
        }
        IpAddr::V6(address) if prefix <= 128 => {
            let mask = u128::MAX.checked_shl(128 - prefix).unwrap_or(0);
            Ok(format!(
                "{}/{prefix}",
                Ipv6Addr::from(u128::from(address) & mask)
            ))
        }
        _ => Err(invalid()),
    }
}

#[cfg(test)]
mod tests;
