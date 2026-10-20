use std::time::Duration;

use herdr_picker_adapters::BlueBubblesSettings;
use serde_json::Value;

use crate::builtin::program::{Ran, ran, run};

mod pages;
mod server;

use pages::pages;
use server::{Server, server};

const UNAVAILABLE: &str = "BlueBubbles is unavailable or returned an invalid response. Check its server and picker configuration.";
const PAGE: usize = 200;

// A BlueBubbles server and its password, read from the Keychain for this run only.
pub(super) struct BlueBubbles {
    server: Server,
    password: String,
}

impl BlueBubbles {
    pub(super) fn connect(settings: &BlueBubblesSettings) -> Result<Self, String> {
        let server = server(&settings.url)?;
        let argv = [
            "security",
            "find-generic-password",
            "-w",
            "-s",
            &settings.keychain_service,
        ];
        let password = run(&argv, Duration::from_secs(5))?.trim().to_string();
        match password.is_empty() {
            true => Err("BlueBubbles server password is empty in the Keychain.".to_string()),
            false => Ok(Self { server, password }),
        }
    }

    // Every record a query finds, a page at a time as the list is read.
    pub(super) fn query<'a>(
        &'a self,
        path: &'a str,
        body: &'a Value,
    ) -> impl Iterator<Item = Result<Value, String>> + 'a {
        pages(move |offset| {
            let mut body = body.clone();
            body["offset"] = offset.into();
            body["limit"] = PAGE.into();
            let args = curl_args(&self.server, &body);
            let argv: Vec<&str> = args.iter().map(String::as_str).collect();
            let config = curl_config(&self.server, path, &self.password);
            answer(&ran(&argv, Some(&config), Duration::from_secs(25))?)
        })
    }
}

pub(super) fn curl_args(server: &Server, body: &Value) -> Vec<String> {
    let proto = if server.local {
        "=http,https"
    } else {
        "=https"
    };
    [
        "curl",
        "--silent",
        "--show-error",
        "--fail",
        "--max-time",
        "20",
        "--max-redirs",
        "0",
        "--proto",
        proto,
        "--header",
        "Content-Type: application/json",
        "--data-binary",
        &body.to_string(),
        "--config",
        "-",
    ]
    .map(String::from)
    .to_vec()
}

// BlueBubbles takes its password in the URL, so the URL goes to curl as config on stdin.
pub(super) fn curl_config(server: &Server, path: &str, password: &str) -> String {
    let encoded: String = password
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                (byte as char).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect();
    format!("url = \"{}/api/v1{path}?password={encoded}\"\n", server.url)
}

pub(super) fn answer(done: &Ran) -> Result<Value, String> {
    if !done.succeeded {
        let status = done
            .stderr
            .split_once("returned error: ")
            .map(|(_, status)| status.trim());
        return Err(match status {
            Some(status) => format!(
                "BlueBubbles request failed (HTTP {status}). Check its URL and server password."
            ),
            None => UNAVAILABLE.to_string(),
        });
    }
    let page: Value = serde_json::from_str(&done.stdout).map_err(|_| UNAVAILABLE.to_string())?;
    match page["data"].is_array() {
        true => Ok(page),
        false => Err("BlueBubbles returned an unexpected response.".to_string()),
    }
}

#[cfg(test)]
mod tests;
