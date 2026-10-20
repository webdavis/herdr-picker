use herdr_picker_domain::{Action, PickerCall, PickerRow, Preview};

use super::command;
use super::inputs::validate;
use crate::builtin::page::{notice, words};

const RECORDS: [&str; 9] = ["A", "AAAA", "CNAME", "MX", "TXT", "NS", "SOA", "SRV", "CAA"];
const MODES: [&str; 5] = ["client", "reverse", "bidirectional", "udp", "server"];

fn choice(label: &str, detail: &str, step: &str) -> PickerRow {
    PickerRow {
        label: label.to_string(),
        preview: Some(Preview::Text(detail.to_string())),
        enter: Action::Next(PickerCall {
            name: step.into(),
            args: vec![label.to_string()],
            state: None,
            query: None,
        }),
        keys: Vec::new(),
        group: None,
    }
}

// The DNS record types; Enter asks for the host to look up.
pub(super) fn dns_records() -> Vec<PickerRow> {
    RECORDS
        .iter()
        .map(|record| {
            let detail =
                format!("dig <host> {record}\nChoose a host and optional DNS server on Enter.");
            choice(record, &detail, "dns-lookup")
        })
        .collect()
}

// The host typed, then an optional server to ask in place of the system's.
pub(super) fn dns_lookup(record: &str, typed: &str) -> Option<(String, PickerRow)> {
    RECORDS.contains(&record).then_some(())?;
    let prompt = "Type the host and an optional DNS server, then press Enter.".to_string();
    let typed: Vec<&str> = typed.split_whitespace().collect();
    let built = || -> Result<String, String> {
        let host = validate("host", typed.first().ok_or("Type the host")?)?;
        let mut argv = vec!["dig".to_string(), "+time=2".into(), "+tries=1".into()];
        if let Some(server) = typed.get(1) {
            argv.push(format!("@{}", validate("host", server)?));
        }
        argv.extend([host, record.to_string()]);
        Ok(joined(&argv))
    };
    let row = match built() {
        Ok(text) => command(&format!("DNS {record}"), &text, ""),
        Err(why) => notice(&why, &format!("dig <host> {record}")),
    };
    Some((prompt, row))
}

// The iperf3 test modes; Enter asks for the server and settings each needs.
pub(super) fn iperf_modes() -> Vec<PickerRow> {
    let detail = "Choose duration, port and host as applicable.\nEnter shows the exact command before preparing it.";
    MODES
        .iter()
        .map(|mode| choice(mode, detail, "iperf-command"))
        .collect()
}

pub(super) fn iperf_command(mode: &str, typed: &str) -> Option<(String, PickerRow)> {
    MODES.contains(&mode).then_some(())?;
    let prompt = match mode {
        "server" => "Type an optional port, then press Enter.",
        "udp" => {
            "Type the iperf3 server, then optionally seconds, port and bitrate, then press Enter."
        }
        _ => "Type the iperf3 server, then optionally seconds and port, then press Enter.",
    };
    let row = match iperf_argv(mode, &typed.split_whitespace().collect::<Vec<_>>()) {
        Ok(argv) => command(&format!("iperf3 {mode}"), &joined(&argv), ""),
        Err(why) => notice(&why, prompt),
    };
    Some((prompt.to_string(), row))
}

fn iperf_argv(mode: &str, typed: &[&str]) -> Result<Vec<String>, String> {
    let given = |at: usize, default: &'static str| typed.get(at).copied().unwrap_or(default);
    if mode == "server" {
        let port = validate("port", given(0, "5201"))?;
        return Ok(strings(&[
            "iperf3",
            "--json",
            "--port",
            &port,
            "--server",
            "--one-off",
        ]));
    }
    let host = validate("host", typed.first().ok_or("Type the iperf3 server")?)?;
    let seconds = validate("duration", given(1, "10"))?;
    let port = validate("port", given(2, "5201"))?;
    let mut argv = strings(&[
        "iperf3", "--json", "--port", &port, "--client", &host, "--time", &seconds,
    ]);
    match mode {
        "reverse" => argv.push("--reverse".into()),
        "bidirectional" => argv.push("--bidir".into()),
        "udp" => argv.extend(strings(&["--udp", "--bitrate", bitrate(given(3, "10M"))?])),
        _ => {}
    }
    Ok(argv)
}

// A rate such as 10M: a whole number, then K, M or G if any.
fn bitrate(text: &str) -> Result<&str, String> {
    let digits = text.strip_suffix(['K', 'M', 'G']).unwrap_or(text);
    let whole = !digits.starts_with('0') && !digits.is_empty();
    match whole && digits.chars().all(|c| c.is_ascii_digit()) {
        true => Ok(text),
        false => Err("Invalid bitrate".to_string()),
    }
}

fn strings(words: &[&str]) -> Vec<String> {
    words.iter().map(|word| word.to_string()).collect()
}

fn joined(argv: &[String]) -> String {
    words(&argv.iter().map(String::as_str).collect::<Vec<_>>())
}

#[cfg(test)]
mod tests;
