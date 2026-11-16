use serde_json::Value;

use super::pane::parsed;
use crate::ProtocolError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaneFront {
    TheShell { folder: Option<String> },
    AnotherProgram,
    NotKnown,
}

pub fn rex_pane_front(process: &str) -> Result<PaneFront, ProtocolError> {
    let process = parsed(process, "process")?;
    let child = process.get("child");
    let foreground = process.get("foreground");
    let pid = |found: Option<&Value>| {
        found
            .and_then(|found| found.get("pid"))
            .and_then(Value::as_u64)
    };
    Ok(match (pid(child), pid(foreground)) {
        (Some(shell), Some(in_front)) if shell == in_front => PaneFront::TheShell {
            folder: foreground
                .and_then(|found| found.get("cwd"))
                .and_then(Value::as_str)
                .map(str::to_string),
        },
        (Some(_), Some(_)) => PaneFront::AnotherProgram,
        _ => PaneFront::NotKnown,
    })
}

#[cfg(test)]
mod tests;
