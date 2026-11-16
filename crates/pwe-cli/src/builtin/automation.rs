use std::time::Duration;

use serde_json::Value;

use super::program::ran;

const SCRIPT: &str = concat!(
    include_str!("automation/contacts.js"),
    include_str!("automation/browsers.js")
);

pub(super) fn automation(request: &Value, timeout: Duration) -> Result<Value, String> {
    let argv = ["osascript", "-l", "JavaScript", "-", &request.to_string()];
    let done = ran(&argv, Some(SCRIPT), timeout)?;
    if !done.succeeded {
        return Err(reason(&done.stderr));
    }
    serde_json::from_str(&done.stdout).map_err(|_| "osascript printed no JSON".to_string())
}

pub(super) fn reason(stderr: &str) -> String {
    let reason = stderr
        .trim()
        .trim_start_matches("execution error: ")
        .trim_start_matches("Error: ");
    reason
        .rsplit_once(" (-")
        .map_or(reason, |(reason, _)| reason)
        .to_string()
}

#[cfg(test)]
mod tests;
