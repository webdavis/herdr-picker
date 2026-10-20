use herdr_picker_domain::PickerRow;

use super::command;
use super::inputs::validate;
use crate::builtin::page::{notice, words};

const TIMINGS: &str = "DNS: %{time_namelookup}s\nConnect: %{time_connect}s\nTLS: %{time_appconnect}s\nFirst byte: %{time_starttransfer}s\nTotal: %{time_total}s\nStatus: %{http_code}\nRemote: %{remote_ip}\n";

// The web checks source: the URL is typed into the search line, and no request runs until a
// prepared command does.
pub(super) fn web_rows(typed: &str, openssl: &str) -> Vec<PickerRow> {
    let typed = typed.trim();
    if typed.is_empty() {
        return vec![notice(
            "Type an HTTP or HTTPS URL",
            "Type an HTTP or HTTPS URL, then choose timing or certificate inspection.\nNo requests run before the prepared command is executed.",
        )];
    }
    match url_rows(typed, openssl) {
        Ok(rows) => rows,
        Err(why) => vec![notice(
            &why,
            "Type an HTTP or HTTPS URL, such as https://example.com.",
        )],
    }
}

fn url_rows(url: &str, openssl: &str) -> Result<Vec<PickerRow>, String> {
    let refused = || "Choose an HTTP or HTTPS URL without credentials".to_string();
    if url.chars().any(char::is_control) {
        return Err("Invalid URL".to_string());
    }
    let (scheme, rest) = url.split_once("://").ok_or_else(refused)?;
    let secure = match scheme.to_lowercase().as_str() {
        "https" => true,
        "http" => false,
        _ => return Err(refused()),
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    if authority.contains('@') {
        return Err(refused());
    }
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) if !authority.ends_with(']') => (
            host,
            Some(
                port.parse::<u16>()
                    .map_err(|_| "Invalid port".to_string())?,
            ),
        ),
        _ => (authority, None),
    };
    let host = host.trim_start_matches('[').trim_end_matches(']');
    if host.is_empty() {
        return Err(refused());
    }
    let host = validate("host", host)?;
    let port = port.unwrap_or(if secure { 443 } else { 80 });
    let curl = words(&[
        "curl",
        "--globoff",
        "--proto",
        "=http,https",
        "--connect-timeout",
        "10",
        "--max-time",
        "30",
        "--silent",
        "--show-error",
        "--output",
        "/dev/null",
        "--dump-header",
        "-",
        "--write-out",
        TIMINGS,
        "--url",
        url,
    ]);
    let mut rows = vec![command(
        "HTTP timings and response headers",
        &curl,
        "Send one request and report phase timings. Redirects are shown without following them.",
    )];
    if secure {
        let check = if host
            .split('%')
            .next()
            .unwrap_or_default()
            .parse::<std::net::IpAddr>()
            .is_ok()
        {
            "-verify_ip"
        } else {
            "-verify_hostname"
        };
        let connect = if host.contains(':') {
            format!("[{host}]:{port}")
        } else {
            format!("{host}:{port}")
        };
        let openssl = words(&[
            openssl,
            "s_client",
            "-connect",
            &connect,
            "-servername",
            &host,
            "-showcerts",
            "-verify_return_error",
            check,
            &host,
        ]);
        rows.push(command("Inspect and verify TLS certificates", &openssl, "Connect explicitly, verify the certificate chain and peer name, and show certificates."));
    }
    Ok(rows)
}

#[cfg(test)]
mod tests;
