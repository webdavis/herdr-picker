use std::fs;
use std::io::Write;
use std::time::Duration;

use super::scan::ScanFiles;
use super::{
    analysis, capture_builder, capture_folder, captures, commands, diagnostics, hosts, interfaces,
    scan_step, ssh_config,
};
use crate::builtin::program::run;
use crate::builtin::rows::RowWriter;
use crate::builtin::{Asked, Places, written};
use crate::command::CommandError;

const QUICK: Duration = Duration::from_secs(3);

// The network picker's steps, or None for a name that is not one of them.
pub(in crate::builtin) fn network_step(
    name: &str,
    arguments: &[&str],
    asked: &Asked,
    places: &Places,
    rows: &mut RowWriter<impl Write>,
) -> Option<Result<(), CommandError>> {
    let typed = asked.query;
    let (prompt, listed) = match (name, arguments) {
        ("network-scan", []) => {
            let ifconfig = run(&["ifconfig"], QUICK).unwrap_or_default();
            let files = ScanFiles::in_folder(&places.captures);
            let prepare = asked.state == Some("prepare");
            let listed =
                scan_step::scan_rows(&interfaces::networks(&ifconfig), typed, prepare, &files);
            let prompt = "Pick a network to scan, or type a range such as 192.168.1.0/24.";
            (Some(prompt.to_string()), listed)
        }
        ("capture-analysis", [path]) => (None, analysis::analysis(path)),
        ("capture-filter", [path]) => (
            Some("Type a Wireshark display filter, then press Enter.".to_string()),
            vec![analysis::filtered(path, typed)],
        ),
        ("network-command", [id]) => one(commands::command_step(id, typed)?),
        ("network-hosts", []) => {
            let home = places.home.as_deref();
            let config = home.map(|home| home.join(".ssh/config"));
            let ssh = config.map_or_else(Vec::new, |config| ssh_config::ssh_hosts(&config, home));
            let etc_hosts = fs::read_to_string("/etc/hosts").ok();
            let tailscale = run(&["tailscale", "status", "--json"], QUICK).ok();
            let listed = hosts::host_rows(&ssh, etc_hosts.as_deref(), tailscale.as_deref());
            (None, listed)
        }
        ("network-dns", []) => (None, diagnostics::dns_records()),
        ("dns-lookup", [record]) => one(diagnostics::dns_lookup(record, typed)?),
        ("network-iperf", []) => (None, diagnostics::iperf_modes()),
        ("iperf-command", [mode]) => one(diagnostics::iperf_command(mode, typed)?),
        ("network-capture", []) => {
            let listed = run(&["ifconfig", "-l"], QUICK).unwrap_or_default();
            (None, capture_builder::capture_interfaces(&listed))
        }
        ("capture-command", [interface]) => one(capture_builder::capture_command(
            interface,
            typed,
            &places.cwd,
            places.home.as_deref(),
        )),
        ("capture-folder", []) => one(capture_folder::folder_rows(
            typed,
            &places.cwd,
            places.home.as_deref(),
        )),
        ("captures-in", [folder]) => {
            let shown = captures::Shown::from_state(asked.state);
            (None, captures::captures(folder.as_ref(), shown))
        }
        _ => return None,
    };
    let header = prompt.map_or(Ok(()), |prompt| rows.header(&prompt));
    Some(written(header.and_then(|()| {
        listed.iter().try_for_each(|row| rows.row(row))
    })))
}

fn one<Row>((prompt, row): (String, Row)) -> (Option<String>, Vec<Row>) {
    (Some(prompt), vec![row])
}

#[cfg(test)]
mod tests;
