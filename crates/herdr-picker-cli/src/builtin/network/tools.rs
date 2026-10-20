use std::net::UdpSocket;
use std::time::Duration;

use crate::builtin::program::ran;
use crate::command::{CommandError, failed};

// herdr-picker network ping <address> <page> | herdr-picker network wake <mac> <broadcast>...
pub(crate) fn network_tool(arguments: &[&str]) -> Result<(), CommandError> {
    match arguments {
        ["ping", address, page] => {
            println!("{page}\n\n{}", pinged(address));
            Ok(())
        }
        ["wake", mac, targets @ ..] if !targets.is_empty() => {
            let packet =
                magic_packet(mac).ok_or_else(|| failed(format!("{mac} is not a MAC address")))?;
            let socket = UdpSocket::bind("0.0.0.0:0").map_err(failed)?;
            socket.set_broadcast(true).map_err(failed)?;
            for target in targets {
                socket.send_to(&packet, (*target, 9)).map_err(failed)?;
            }
            // A sent packet proves nothing about the device; only one set up for it wakes.
            println!(
                "Wake-up packet sent to {mac}. Reopen the devices to see whether it answers; only devices set up for Wake-on-LAN will."
            );
            Ok(())
        }
        _ => Err(CommandError::Usage),
    }
}

// How long the device took to answer one ping, and the name the system resolver has for it.
fn pinged(address: &str) -> String {
    let ping = match address.contains(':') {
        true => ran(
            &["ping6", "-c", "1", address],
            None,
            Duration::from_millis(1500),
        ),
        false => ran(
            &["ping", "-c", "1", "-W", "1000", address],
            None,
            Duration::from_millis(1500),
        ),
    };
    let named = ran(
        &["dscacheutil", "-q", "host", "-a", "ip_address", address],
        None,
        Duration::from_secs(2),
    );
    let mut lines = vec![format!(
        "Pinged    {}",
        ping_time(&ping.map(|done| done.stdout).unwrap_or_default())
    )];
    lines.extend(
        named
            .ok()
            .and_then(|done| resolved(&done.stdout))
            .map(|name| format!("Hostname  {name}")),
    );
    lines.join("\n")
}

pub(super) fn ping_time(output: &str) -> String {
    let time = output
        .split_once("time=")
        .and_then(|(_, rest)| rest.split_once(" ms"));
    match time.and_then(|(time, _)| time.parse::<f64>().ok()) {
        Some(time) => format!("{time:.0} ms"),
        None => "no reply".to_string(),
    }
}

pub(super) fn resolved(output: &str) -> Option<String> {
    output
        .lines()
        .find_map(|line| line.strip_prefix("name: "))
        .map(str::to_string)
}

// Six 0xff bytes, then the MAC sixteen times.
pub(super) fn magic_packet(mac: &str) -> Option<Vec<u8>> {
    let bytes: Vec<u8> = mac
        .split([':', '-'])
        .map(|part| u8::from_str_radix(part, 16).ok())
        .collect::<Option<_>>()?;
    (bytes.len() == 6).then(|| [vec![0xff; 6], bytes.repeat(16)].concat())
}

#[cfg(test)]
mod tests;
