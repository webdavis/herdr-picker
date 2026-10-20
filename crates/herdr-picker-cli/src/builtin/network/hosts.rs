use herdr_picker_domain::{Action, PickerRow, Preview};
use herdr_picker_protocol::shell_word;
use serde_json::Value;

use super::command;
use super::ssh_config::SshHost;
use crate::builtin::page::{fields, notice, words};
use crate::builtin::paint::dim;

const TAILSCALE: [&str; 8] = [
    "HostName",
    "DNSName",
    "TailscaleIPs",
    "OS",
    "Online",
    "LastSeen",
    "Relay",
    "CurAddr",
];

// Saved hosts: SSH aliases prepare ssh, hosts-file names and Tailscale peers paste their address.
pub(super) fn host_rows(
    ssh: &[SshHost],
    etc_hosts: Option<&str>,
    tailscale: Option<&str>,
) -> Vec<PickerRow> {
    let mut rows: Vec<PickerRow> = ssh.iter().map(ssh_row).collect();
    match etc_hosts {
        Some(text) => rows.extend(text.lines().flat_map(hosts_line)),
        None => rows.push(notice("/etc/hosts is unavailable", "")),
    }
    let status = tailscale.and_then(|text| serde_json::from_str::<Value>(text).ok());
    match status {
        Some(status) => rows.extend(peers(&status)),
        None => rows.push(notice("Tailscale status is unavailable", "")),
    }
    rows
}

fn ssh_row(host: &SshHost) -> PickerRow {
    let mut purpose = vec![format!(
        "Static host alias from {}; SSH evaluates the full configuration when run.",
        host.file.display()
    )];
    purpose.extend(host.settings.iter().cloned());
    let text = words(&["ssh", &host.alias]);
    PickerRow {
        label: format!("{}  {}", host.alias, dim("SSH")),
        group: Some("SSH".to_string()),
        ..command(&host.alias, &text, &purpose.join("\n"))
    }
}

fn hosts_line(line: &str) -> Vec<PickerRow> {
    let words: Vec<&str> = line
        .split('#')
        .next()
        .unwrap_or_default()
        .split_whitespace()
        .collect();
    let Some((address, names)) = words.split_first() else {
        return Vec::new();
    };
    names
        .iter()
        .map(|name| {
            let page = fields(&[("Name", name.to_string()), ("IP", address.to_string())]);
            pasted(name, address, page, "/etc/hosts")
        })
        .collect()
}

fn peers(status: &Value) -> Vec<PickerRow> {
    let others = status["Peer"]
        .as_object()
        .into_iter()
        .flat_map(|peers| peers.values());
    std::iter::once(&status["Self"])
        .chain(others)
        .filter_map(|peer| {
            let name = ["DNSName", "HostName", "ID"]
                .iter()
                .filter_map(|key| peer[key].as_str())
                .find(|name| !name.is_empty())?;
            let address = peer["TailscaleIPs"][0].as_str().unwrap_or_default();
            let shown: Vec<(&str, String)> = TAILSCALE
                .iter()
                .map(|key| (*key, text(&peer[key])))
                .collect();
            Some(pasted(name, address, fields(&shown), "Tailscale"))
        })
        .collect()
}

fn text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Array(items) => items.iter().map(text).collect::<Vec<_>>().join(", "),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

// Enter pastes the address, or the name when there is none.
fn pasted(name: &str, address: &str, page: String, group: &str) -> PickerRow {
    let label = match address.is_empty() {
        true => name.to_string(),
        false => format!("{name}  {}", dim(address)),
    };
    PickerRow {
        label,
        preview: Some(Preview::Text(page)),
        enter: Action::Insert(shell_word(if address.is_empty() { name } else { address })),
        keys: Vec::new(),
        group: Some(group.to_string()),
    }
}

#[cfg(test)]
mod tests;
