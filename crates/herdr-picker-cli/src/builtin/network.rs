use std::env;
use std::io::Write;

use herdr_picker_adapters::{on_path, read_picker_config};
use herdr_picker_domain::{Action, PickerRow, Preview};

use super::clock::{local_offset, now};
use super::program::run;
use super::rows::{RowWriter, key};
use super::{Asked, Places, written};
use crate::command::{CommandError, picker_config_path};

mod analysis;
mod bonjour;
mod captures;
mod commands;
mod controller;
mod device_page;
mod device_row;
mod devices;
mod gather;
mod inputs;
mod interfaces;
mod naming;
mod scan;
mod scan_step;
mod sightings;
mod tools;
mod unifi;
mod unifi_record;
mod web;

use device_page::Clock;
use scan::ScanFiles;
pub(crate) use tools::network_tool;

pub(super) fn network(
    source: &str,
    asked: &Asked,
    places: &Places,
    rows: &mut RowWriter<impl Write>,
) -> Result<(), CommandError> {
    let listed = match source {
        "captures" => captures::captures(&places.cwd, captures::Shown::from_state(asked.state)),
        "web checks" => {
            let openssl = on_path("openssl", env::var_os("PATH").as_deref());
            let openssl = openssl.map_or("openssl".to_string(), |path| {
                path.to_string_lossy().into_owned()
            });
            web::web_rows(asked.query, &openssl)
        }
        "commands" => commands::command_rows(),
        _ => {
            let unifi = read_picker_config(&picker_config_path())
                .ok()
                .and_then(|config| config.unifi);
            let home = places
                .home
                .as_ref()
                .map(|home| home.to_string_lossy().into_owned());
            let clock = Clock {
                now: now(),
                offset: local_offset(),
            };
            gather::device_rows(
                unifi.as_ref(),
                home.as_deref(),
                &ScanFiles::in_folder(&places.captures),
                clock,
            )
        }
    };
    written(listed.iter().try_for_each(|row| rows.row(row)))
}

// The network picker's steps, or None for a name that is not one of them.
pub(super) fn network_step(
    name: &str,
    arguments: &[&str],
    asked: &Asked,
    places: &Places,
    rows: &mut RowWriter<impl Write>,
) -> Option<Result<(), CommandError>> {
    let (prompt, listed) = match (name, arguments) {
        ("network-scan", []) => {
            let ifconfig =
                run(&["ifconfig"], std::time::Duration::from_secs(3)).unwrap_or_default();
            let files = ScanFiles::in_folder(&places.captures);
            let listed = scan_step::scan_rows(
                &interfaces::networks(&ifconfig),
                asked.query,
                asked.state == Some("prepare"),
                &files,
            );
            (
                Some("Pick a network to scan, or type a range such as 192.168.1.0/24.".to_string()),
                listed,
            )
        }
        ("capture-analysis", [path]) => (None, analysis::analysis(path)),
        ("capture-filter", [path]) => (
            Some("Type a Wireshark display filter, then press Enter.".to_string()),
            vec![analysis::filtered(path, asked.query)],
        ),
        ("network-command", [id]) => {
            let (prompt, row) = commands::command_step(id, asked.query)?;
            (Some(prompt), vec![row])
        }
        _ => return None,
    };
    let header = prompt.map_or(Ok(()), |prompt| rows.header(&prompt));
    Some(written(header.and_then(|()| {
        listed.iter().try_for_each(|row| rows.row(row))
    })))
}

// A network command: Enter prepares it, Ctrl-Enter runs it now, Alt-Y copies it.
fn command(title: &str, command: &str, purpose: &str) -> PickerRow {
    let detail = [title, command, purpose, "Enter prepares the command."];
    let detail: Vec<&str> = detail.into_iter().filter(|line| !line.is_empty()).collect();
    PickerRow {
        label: title.to_string(),
        preview: Some(Preview::Text(detail.join("\n"))),
        enter: Action::Insert(command.to_string()),
        keys: vec![
            (key("ctrl-enter"), Action::Run(command.to_string())),
            (key("alt-y"), Action::Copy(command.to_string())),
        ],
        group: None,
    }
}
