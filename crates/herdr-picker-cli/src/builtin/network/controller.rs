use herdr_picker_adapters::UnifiSettings;

use crate::path::expand_tilde;

const API: &str = "/proxy/network/integration/v1";

// How curl reaches the controller: the URL it asks, and where it connects when the
// certificate names another host than the address the controller answers at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Controller {
    pub(super) base: String,
    pub(super) connect_to: Option<String>,
    pub(super) ca_file: Option<String>,
}

pub(super) fn controller(
    settings: &UnifiSettings,
    home: Option<&str>,
) -> Result<Controller, String> {
    let refused = || "UniFi requires an HTTPS controller URL without credentials".to_string();
    let rest = settings.url.strip_prefix("https://").ok_or_else(refused)?;
    if rest.contains(['@', '?', '#']) {
        return Err(refused());
    }
    let (authority, path) = rest
        .split_once('/')
        .map_or((rest, ""), |(authority, path)| (authority, path));
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) if !authority.ends_with(']') => {
            (host, port.parse::<u16>().map_err(|_| refused())?)
        }
        _ => (authority, 443),
    };
    if host.is_empty() {
        return Err(refused());
    }
    let named = settings
        .server_name
        .as_deref()
        .filter(|name| !name.is_empty());
    let path = path.trim_end_matches('/');
    let path = if path.is_empty() {
        String::new()
    } else {
        format!("/{path}")
    };
    let ca_file = settings
        .ca_file
        .as_deref()
        .map(|file| expand_tilde(file, home).map(|file| file.to_string_lossy().into_owned()))
        .transpose()?;
    Ok(Controller {
        base: format!("https://{}:{port}{path}{API}", named.unwrap_or(host)),
        connect_to: named.map(|name| format!("{name}:{port}:{host}:{port}")),
        ca_file,
    })
}

// The key goes to curl on stdin, so no process listing ever shows it.
pub(super) fn curl_args(controller: &Controller, path: &str, offset: usize) -> Vec<String> {
    let mut args: Vec<String> = [
        "curl",
        "--silent",
        "--show-error",
        "--fail",
        "--max-time",
        "4",
        "--max-redirs",
        "0",
        "--proto",
        "=https",
        "--header",
        "@-",
    ]
    .map(String::from)
    .to_vec();
    if let Some(file) = &controller.ca_file {
        args.extend(["--cacert".to_string(), file.clone()]);
    }
    if let Some(target) = &controller.connect_to {
        args.extend(["--connect-to".to_string(), target.clone()]);
    }
    args.extend([
        "--url".to_string(),
        format!("{}{path}?offset={offset}&limit=200", controller.base),
    ]);
    args
}

#[cfg(test)]
mod tests;
