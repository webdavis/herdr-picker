use herdr_picker_domain::PickerRow;

use super::captures::next;
use super::command;
use super::inputs::validate;
use crate::builtin::page::{notice, words};

// The ways to read one capture.
pub(super) fn analysis(path: &str) -> Vec<PickerRow> {
    let tshark = |extra: &[&str]| words(&[&["tshark", "-n", "-r", path][..], extra].concat());
    let filter = format!("{} -Y <display-filter>", tshark(&[]));
    vec![
        command("Capture file summary", &words(&["capinfos", path]), ""),
        command("Packets (numeric addresses)", &tshark(&[]), ""),
        command(
            "TCP and UDP conversations",
            &tshark(&["-q", "-z", "conv,tcp", "-z", "conv,udp"]),
            "",
        ),
        command("Protocol hierarchy", &tshark(&["-q", "-z", "io,phs"]), ""),
        PickerRow {
            enter: next("capture-filter", path),
            keys: Vec::new(),
            ..command(
                "Packets matching a display filter",
                &filter,
                "Choose a Wireshark display filter on Enter. Capture filters use -f; this saved-file view uses -Y.",
            )
        },
    ]
}

// The display filter is typed into the step's search line.
pub(super) fn filtered(path: &str, typed: &str) -> PickerRow {
    match validate("display filter", typed) {
        Ok(filter) => command(
            "Filtered capture",
            &words(&["tshark", "-n", "-r", path, "-Y", &filter]),
            "",
        ),
        Err(_) => notice(
            "Type a Wireshark display filter, such as tcp.port == 443",
            "The command shows here once a filter is typed.",
        ),
    }
}

#[cfg(test)]
mod tests;
