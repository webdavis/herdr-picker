use std::path::{Path, PathBuf};

use herdr_picker_domain::{Action, PickerRow, Preview};

use super::device_page::Clock;
use super::sightings::Sighting;
use crate::builtin::clock::stamp;
use crate::builtin::page::words;
use crate::builtin::paint::{blue, bold, dim};
use crate::builtin::rows::key;

// A scan is remembered for a day, so the devices source still shows it after the picker closes.
const KEPT: i64 = 24 * 3600;

// Where Ctrl-X's nmap run leaves its results: XML for the rows, its own report for people.
pub(super) struct ScanFiles {
    pub(super) xml: PathBuf,
    pub(super) report: PathBuf,
}

impl ScanFiles {
    pub(super) fn in_folder(folder: &Path) -> Self {
        Self {
            xml: folder.join("network-scan.xml"),
            report: folder.join("network-scan.txt"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct Scan {
    pub(super) network: String,
    pub(super) hosts: Vec<Sighting>,
    pub(super) at: i64,
    pub(super) report: String,
}

// nmap's XML: the range from its command line, when it ran, and each host's addresses and name.
pub(super) fn parse_scan(xml: &str, report: &str, now: i64) -> Option<Scan> {
    let run = xml.split_once("<nmaprun ")?.1;
    let at: i64 = attribute(run, "start")?.parse().ok()?;
    let network = attribute(run, "args")?
        .split_whitespace()
        .last()?
        .to_string();
    let hosts = xml.split("</host>").filter_map(host).collect();
    (now - at < KEPT).then(|| Scan {
        network,
        hosts,
        at,
        report: report.trim().to_string(),
    })
}

fn host(element: &str) -> Option<Sighting> {
    let mut seen = Sighting::default();
    for address in element.split("<address ").skip(1) {
        match attribute(address, "addrtype").as_deref() {
            Some("mac") => {
                seen.mac = attribute(address, "addr").unwrap_or_default();
                seen.vendor = attribute(address, "vendor").unwrap_or_default();
            }
            _ => seen.ip = attribute(address, "addr").unwrap_or_default(),
        }
    }
    let name = element
        .split_once("<hostname ")
        .and_then(|(_, tag)| attribute(tag, "name"));
    seen.name = name.unwrap_or_default();
    (!seen.ip.is_empty()).then_some(seen)
}

fn attribute(tag: &str, name: &str) -> Option<String> {
    let tag = format!(" {}", tag.split_once('>').map_or(tag, |(tag, _)| tag));
    let value = tag.split_once(&format!(" {name}=\""))?.1.split_once('"')?.0;
    Some(
        value
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&apos;", "'")
            .replace("&amp;", "&"),
    )
}

pub(super) fn scan_command(range: &str) -> String {
    let six = if range.contains(':') {
        vec!["-6"]
    } else {
        vec![]
    };
    words(&[&["sudo", "nmap"][..], &six, &["-sn", range]].concat())
}

// The last scan: its report in the preview and on Alt-Y, its command on Enter.
pub(super) fn scan_row(scan: &Scan, clock: Clock) -> PickerRow {
    let (when, today) = (stamp(scan.at, clock.offset), stamp(clock.now, clock.offset));
    let when = if when[..10] == today[..10] {
        when[11..].to_string()
    } else {
        when
    };
    let count = scan.hosts.len();
    let summary = format!(
        "{count} host{} up at {when}",
        if count == 1 { "" } else { "s" }
    );
    let command = scan_command(&scan.network);
    PickerRow {
        label: format!(
            "{}  {}  {}",
            bold("Last scan"),
            blue(&scan.network),
            dim(&summary)
        ),
        preview: Some(Preview::Text(format!(
            "Last scan of {}\n\nnmap found {summary}.\n\n{}",
            scan.network, scan.report
        ))),
        enter: Action::Insert(command.clone()),
        keys: vec![
            (key("ctrl-enter"), Action::Run(command)),
            (key("alt-y"), Action::Copy(scan.report.clone())),
        ],
        group: Some("Scan".to_string()),
    }
}

#[cfg(test)]
mod tests;
