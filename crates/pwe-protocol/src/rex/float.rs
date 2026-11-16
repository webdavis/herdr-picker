use pwe_domain::{FLOAT_LABEL, preview_popup_size};
use serde_json::json;

use super::KeyFacts;

const TERMINAL_FLAVOR: &str = "com.superlogical.terminal.shell";

pub fn float_request(facts: &KeyFacts, command: &[String]) -> String {
    let window = facts.window;
    let float = preview_popup_size(window);
    let width = float.columns as f64 / window.columns as f64;
    let height = float.rows as f64 / window.rows as f64;
    json!({
        "session_id": facts.session,
        "focus": true,
        "bounds": [(1.0 - width) / 2.0, (1.0 - height) / 2.0, width, height],
        "layout": {"block": {
            "flavor": TERMINAL_FLAVOR,
            "label": FLOAT_LABEL,
            "options": {
                "cwd": facts.folder,
                "size": {"columns": float.columns, "rows": float.rows},
                "command": command,
            },
        }},
    })
    .to_string()
}

#[cfg(test)]
mod tests;
