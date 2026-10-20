use std::collections::BTreeMap;
use std::path::Path;

use herdr_picker_domain::{Action, PickerCall, PickerRow, Preview};

use super::command;
use super::inputs::validate;
use crate::builtin::page::{notice, words};

const SETTINGS: [&str; 7] = ["host", "proto", "port", "count", "seconds", "tool", "file"];
const PROTOCOLS: [&str; 5] = ["any", "tcp", "udp", "icmp", "icmp6"];
const PROMPT: &str = "Type settings such as host=nas.local proto=tcp port=443 count=50 seconds=10 tool=tcpdump file=out.pcap, then press Enter.";

// The interfaces `ifconfig -l` names; Enter asks for the capture's settings.
pub(super) fn capture_interfaces(listed: &str) -> Vec<PickerRow> {
    let mut interfaces: Vec<&str> = listed.split_whitespace().collect();
    if interfaces.is_empty() {
        interfaces.push("en0");
    }
    interfaces
        .into_iter()
        .map(|interface| PickerRow {
            label: format!("Capture on {interface}"),
            preview: Some(Preview::Text("Choose packet count, optional duration, output and capture filter on Enter.\nThe final command is shown for confirmation.".to_string())),
            enter: Action::Next(PickerCall {
                name: "capture-command".into(),
                args: vec![interface.to_string()],
                state: None,
                query: None,
            }),
            keys: Vec::new(),
            group: None,
        })
        .collect()
}

// The capture command for the settings typed; each one left out keeps its default.
pub(super) fn capture_command(
    interface: &str,
    typed: &str,
    cwd: &Path,
    home: Option<&Path>,
) -> (String, PickerRow) {
    let row = match settings(typed).and_then(|set| argv(interface, &set, cwd, home)) {
        Ok(argv) => {
            let argv: Vec<&str> = argv.iter().map(String::as_str).collect();
            command("Packet capture", &words(&argv), "")
        }
        Err(why) => notice(&why, PROMPT),
    };
    (PROMPT.to_string(), row)
}

fn settings(typed: &str) -> Result<BTreeMap<&str, &str>, String> {
    typed
        .split_whitespace()
        .map(|word| {
            let (name, value) = word.split_once('=').ok_or("Type settings as name=value")?;
            match SETTINGS.contains(&name) {
                true => Ok((name, value)),
                false => Err(format!("Unknown setting {name}")),
            }
        })
        .collect()
}

fn argv(
    interface: &str,
    set: &BTreeMap<&str, &str>,
    cwd: &Path,
    home: Option<&Path>,
) -> Result<Vec<String>, String> {
    let given = |name| set.get(name).copied().filter(|value| !value.is_empty());
    let tool = given("tool").unwrap_or("tshark");
    if !["tshark", "tcpdump"].contains(&tool) {
        return Err("Choose tshark or tcpdump".to_string());
    }
    let protocol = given("proto").unwrap_or("any");
    if !PROTOCOLS.contains(&protocol) {
        return Err("Choose any, tcp, udp, icmp or icmp6".to_string());
    }
    let mut clauses = Vec::new();
    if let Some(host) = given("host") {
        clauses.push(format!("host {}", validate("host", host)?));
    }
    if protocol != "any" {
        clauses.push(protocol.to_string());
    }
    if let Some(port) = given("port") {
        clauses.push(format!("port {}", validate("port", port)?));
    }
    let interface = validate("interface", interface)?;
    let count = validate("count", given("count").unwrap_or("20"))?;
    let mut argv: Vec<String> = ["sudo", tool, "-n", "-i", &interface, "-c", &count]
        .map(String::from)
        .to_vec();
    if let Some(seconds) = given("seconds") {
        if tool == "tcpdump" {
            return Err("Use tshark for a duration-limited capture".to_string());
        }
        argv.extend([
            "-a".into(),
            format!("duration:{}", validate("duration", seconds)?),
        ]);
    }
    if let Some(file) = given("file") {
        let path = match (file.strip_prefix("~/"), home) {
            (Some(inside), Some(home)) => home.join(inside),
            _ => cwd.join(file),
        };
        if path.exists() {
            return Err("Capture output already exists; choose a new filename".to_string());
        }
        argv.extend(["-w".into(), validate("output", &path.to_string_lossy())?]);
    }
    if !clauses.is_empty() {
        if tool == "tshark" {
            argv.push("-f".into());
        }
        argv.push(clauses.join(" and "));
    }
    Ok(argv)
}

#[cfg(test)]
mod tests;
