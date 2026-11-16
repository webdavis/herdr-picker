use std::path::Path;

use pwe_domain::{Action, PickerRow, Preview};

use super::inputs::validate;
use super::interfaces::Network;
use super::scan::{ScanFiles, scan_command};
use crate::builtin::page::{notice, words};
use crate::builtin::paint::{blue, dim};

fn asking_for_the_password_every_time(command: &str) -> String {
    command.replacen("sudo nmap", "sudo -k nmap", 1)
}

fn run_scan(range: &str, files: &ScanFiles) -> String {
    let folder = files.xml.parent().unwrap_or(Path::new("."));
    let xml = files.xml.to_string_lossy();
    let report = files.report.to_string_lossy();
    let nmap = asking_for_the_password_every_time(&scan_command(range));
    format!(
        "mkdir -p {} && {nmap} -oX {} -oN {}",
        words(&[&folder.to_string_lossy()]),
        words(&[&xml]),
        words(&[&report])
    )
}

pub(super) fn scan_rows(
    networks: &[Network],
    typed: &str,
    prepare: bool,
    files: &ScanFiles,
) -> Vec<PickerRow> {
    let enter = |range: &str| match prepare {
        true => Action::Insert(scan_command(range)),
        false => Action::Run(run_scan(range, files)),
    };
    let says = if prepare {
        "Enter prepares the nmap command."
    } else {
        "Enter scans it with nmap, which asks for your password."
    };
    let mut rows = Vec::new();
    match validate("cidr", typed) {
        _ if typed.trim().is_empty() => {}
        Ok(range) => rows.push(PickerRow {
            label: format!("Scan {}", blue(&range)),
            preview: Some(Preview::Text(format!("Network: {range}\n{says}"))),
            enter: enter(&range),
            keys: Vec::new(),
            group: None,
        }),
        Err(why) => rows.push(notice(
            &why,
            "Type the range in the search box, such as 192.168.1.0/24.",
        )),
    }
    rows.extend(networks.iter().map(|network| {
        let range = network.range();
        PickerRow {
            label: format!("{}  {}", blue(&range), dim(&network.interface)),
            preview: Some(Preview::Text(format!(
                "Network: {range}\nInterface: {}\n{says}",
                network.interface
            ))),
            enter: enter(&range),
            keys: Vec::new(),
            group: None,
        }
    }));
    if typed.trim().is_empty() {
        rows.push(notice(
            "Type a range by hand",
            "Type the range in the search box, such as 192.168.1.0/24.",
        ));
    }
    rows
}

#[cfg(test)]
mod tests;
